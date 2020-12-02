use crate::db::{keys, Batch, DBSync, DB};
use crate::prelude::*;
use crate::utils::drop_map::DropMap;
use crate::{options, sync};
use crossbeam::sync::ShardedLockReadGuard;
use rand::{rngs::ThreadRng, Rng};

sync!(ContextSync(Context) {});

options!(ContextOptionsBuilder(ContextOptions) {
    /// Directory to be used to store the data.
    path: String = None,
    /// How long should we wait to drop a editor with no active connections in ms.
    /// (Default: 3 Min)
    editor_ttl: Timestamp = Some(180 * 1000),
    /// Number of open editors we are allowed to keep until triggering the GC.
    /// (Default: 64)
    editors_cache_capacity: usize = Some(64),
    /// Maximum number of editor we're allowed to have, 0 means unlimited.
    max_number_of_editors: usize = Some(0)
});

options!(CreateBranchOptionsBuilder(CreateBranchOptions) {
    /// Head of the commit.
    head: CommitIdentifier = None,
    /// Name of the branch. (Required)
    title: String = None,
    /// The user who created the commit. (Required)
    user: UserId = None,
    /// Is it an static branch? (No live changes.)
    is_static: bool = Some(false),
    /// Is this branch archived? (No further change is allowed.)
    is_archived: bool = Some(false)
});

/// Context is the central controller of each project, usually there is only one
/// initiated instance of Context in an entire project.
pub struct Context {
    db: DBSync,
    editors: DropMap<BranchIdentifier, EditorSync>,
    rng: ThreadRng,
}

impl Context {
    /// Create a new context using the given options.
    pub fn new(options: &ContextOptions) -> Self {
        Self {
            db: DBSync::new(DB::open(&options.path)),
            editors: DropMap::new(options.editors_cache_capacity, options.editor_ttl),
            rng: rand::thread_rng(),
        }
    }

    /// Create and initialize a new repository owned by the given user.
    pub fn create_repository(&mut self, user: UserId) -> Result<RepositoryId> {
        let repository_id = {
            let db = self.db.read()?;
            loop {
                let id = self.rng.gen::<RepositoryId>();
                if db.get_partial(keys::RepositoryExist(id))?.is_none() {
                    break id;
                }
            }
        };
        let time = now();
        let mut batch = Batch::new();
        batch.put(
            keys::Repository(repository_id),
            &RepositoryInfo {
                user,
                time,
                fork_of: None,
            },
        );
        batch.push(keys::Log(repository_id), &LogEvent::Init { user, time });
        let mut branch = BranchInfo::init(time, user);
        let branch_uuid = branch.hash();
        let commit = CommitInfo::init(
            BranchIdentifier {
                repository: repository_id,
                uuid: branch_uuid,
            },
            time,
            user,
        );
        let commit_id =
            commit.write_commit(&mut batch, &Snapshot::default(), &CompactDelta::default());
        branch.head = commit_id;
        branch.write_branch(&mut batch, repository_id, Some(branch_uuid));
        self.db.write()?.perform(batch)?;
        Ok(repository_id)
    }

    /// Create a new branch by forking another head.
    pub fn create_branch(&mut self, options: CreateBranchOptions) -> Result<BranchIdentifier> {
        let repository = options.head.repository;
        let head_origin = {
            self.db
                .read()?
                .get_partial(keys::CommitOrigin(options.head))?
                .ok_or(Error::CommitNotFound)?
        };

        let fork_point: ForkPoint = Some((head_origin.branch, options.head));

        let branch = BranchInfo {
            head: options.head,
            fork_point,
            created_at: now(),
            user: options.user,
            is_static: options.is_static,
            is_archived: options.is_archived,
            title: options.title,
        };

        let mut batch = Batch::new();
        let id = branch.write_branch(&mut batch, repository, None);
        self.db.write()?.perform(batch)?;
        Ok(id)
    }

    /// Open a new editor on the given branch.
    pub fn open_editor(
        &mut self,
        sync: &ContextSync,
        branch_id: BranchIdentifier,
        user: UserId,
    ) -> Result<EditorSync> {
        // Just to make borrow-checker happy.
        let (editors, db) = (&mut self.editors, &mut self.db);

        let open_editor = || {
            // All of the db reads are placed in one scope so we drop the read-lock faster.
            let (info, mut snapshot, live_changes) = {
                let db = db.read()?;
                let info = db
                    .get(keys::Branch(branch_id))?
                    .ok_or(Error::BranchNotFound)?;
                let commit_snapshot = checkout(&db, info.head)?;
                let live_changes = db.get(keys::LiveChanges(branch_id))?.unwrap_or(Vec::new());
                (info, commit_snapshot, live_changes)
            };

            for batch in &live_changes {
                snapshot
                    .apply_batch_patch(&batch.patches, false)
                    .map_err(|_| Error::CheckoutFailed)?;
            }

            let editor = Editor {
                db: db.clone(),
                snapshot,
                id: branch_id,
                live_changes,
                info,
            };

            Ok(EditorSync::new(editor, branch_id, sync.clone(), None))
        };

        editors
            .get_or_maybe_insert_with(branch_id, open_editor)
            .map(|x| x.open(user))
    }

    /// Internal method, called by EditorSync to inform us that a branch is dropped.
    pub(crate) fn drop_editor(&mut self, branch: BranchIdentifier) {
        self.editors.drop(branch, now());
    }
}

#[inline]
fn checkout(db: &ShardedLockReadGuard<DB>, head: CommitIdentifier) -> Result<Snapshot> {
    // TODO(qti3e) This method needs to be improved:
    // 1. Use SnapshotRef instead of snapshot.
    // 2. If the snapshot is not available (might be lost) compute it from the delta.
    // 3. LFU cache.
    db.get(keys::CommitSnapshot(head))?
        .ok_or(Error::CheckoutFailed)
}
