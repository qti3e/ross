use crate::db::{DBSync, DB};
use crate::prelude::*;
use crate::utils::drop_map::DropMap;
use crate::{options, sync};
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
    head: Option<CommitIdentifier> = Some(None),
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
    pub fn create_repository(&mut self, _user: UserId) -> Result<RepositoryId> {
        let id = self.rng.gen::<RepositoryId>();
        // unimplemented!();
        Ok(id)
    }

    /// Create a new branch by forking another head.
    pub fn create_branch(&mut self, _options: CreateBranchOptions) -> Result<BranchIdentifier> {
        unimplemented!()
    }

    /// Open a new editor on the given branch.
    pub fn open_editor(&mut self, branch: BranchIdentifier, user: UserId) -> Result<EditorSync> {
        self.editors
            .get_or_maybe_insert_with(branch, || unimplemented!())
            .map(|x| x.open(user))
    }

    /// Internal method, called by EditorSync to inform us that a branch is dropped.
    pub(crate) fn drop_editor(&mut self, branch: BranchIdentifier) {
        self.editors.drop(branch, now());
    }
}
