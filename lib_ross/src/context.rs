use crate::branch::{BranchIdentifier, BranchInfo};
use crate::commit::CommitIdentifier;
use crate::db::{data as D, Batch, DBSync, DB};
use crate::drop_map::DropMap;
use crate::error::Result;
use crate::session::SessionSync;
use crate::snapshot::Snapshot;
use crate::sync;
use crate::Timestamp;
use lfu::LFUCache;

sync!(sync ContextSync(Context) {});

pub struct Context {
    db: DBSync,
    opts: ContextOptions,
    snapshot_cache: LFUCache<CommitIdentifier, Snapshot>,
    sessions: DropMap<BranchIdentifier, SessionSync>,
}

impl Context {
    pub fn new(options: ContextOptions) -> Self {
        let path = options.path.clone().unwrap();

        Context {
            db: DBSync::new(DB::open(&path)),
            opts: options.clone(),
            snapshot_cache: LFUCache::with_capacity(options.snapshot_cache_capacity).unwrap(),
            sessions: DropMap::new(options.session_drop_queue_capacity),
        }
    }

    /// Return the snapshot of a commit.
    pub fn snapshot(&mut self, commit: CommitIdentifier) -> Result<Snapshot> {
        if let Some(snapshot) = self.snapshot_cache.get(&commit) {
            return Ok(snapshot.clone());
        }

        let snapshot = {
            let db = self.db.read()?;
            match db.get(D::SnapshotKey(commit.clone()))? {
                Some(snapshot) => snapshot,
                None => Snapshot::default(),
            }
        };

        self.snapshot_cache.set(commit, snapshot.clone());
        Ok(snapshot)
    }

    /// Create a new branch in a repository with the given information.
    pub fn create_branch(&self, id: BranchIdentifier, info: BranchInfo) -> Result<()> {
        let mut batch = Batch::new();
        batch.append(
            D::BranchesKey(id.repository),
            D::BranchesAppendItem(id.uuid),
        );
        batch.put(D::BranchInfoKey(id), &D::BranchInfoValue(info));
        self.db.write()?.perform(batch)
    }

    /// This method is called by a `SessionSync` to indicate that it was dropped
    /// and the number of strong references has reached `2`, in that case this
    /// method adds the session to a watch list, and waits at least `self.ttl` ms
    /// before closing the session.
    ///
    /// Why `2`? When this method is called `SessionSync` has not yet actually
    /// dropped the Arc, so it still owns a reference to it (1 ref), we also keep
    /// a version of the session here in the `self.sessions` map (1 more ref) so
    /// reaching `2` basically means that there is no other active session.
    pub(crate) fn drop_session(&mut self, id: &BranchIdentifier) {
        let opts = &self.opts;
        let expiration = opts.session_ttl + crate::now();
        self.sessions.drop(id.clone(), expiration);
    }
}

#[derive(Debug, Clone)]
pub struct ContextOptions {
    pub path: Option<String>,
    pub snapshot_cache_capacity: usize,
    pub session_ttl: Timestamp,
    pub session_drop_queue_capacity: usize,
}

impl Default for ContextOptions {
    fn default() -> Self {
        ContextOptions {
            path: None,
            snapshot_cache_capacity: 128,
            session_ttl: 150 * 1000, // 2.5 min,
            session_drop_queue_capacity: 64,
        }
    }
}
