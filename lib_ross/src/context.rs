use crate::branch::{BranchIdentifier, BranchInfo};
use crate::commit::CommitIdentifier;
use crate::db::{data as D, Batch, DBSync, DB};
use crate::error::Result;
use crate::session::Session;
use crate::snapshot::Snapshot;
use lfu::LFUCache;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Context {
    db: DBSync,
    snapshot_cache: LFUCache<CommitIdentifier, Snapshot>,
    active_sessions: HashMap<BranchIdentifier, Arc<Session>>,
}

impl Context {
    pub fn new(options: &ContextOptions) -> Self {
        let path = options.path.clone().unwrap();
        Context {
            db: DBSync::new(DB::open(&path)),
            snapshot_cache: LFUCache::with_capacity(options.snapshot_lfu_capacity).unwrap(),
            active_sessions: HashMap::with_capacity(64),
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
}

pub struct ContextOptions {
    pub path: Option<String>,
    pub snapshot_lfu_capacity: usize,
}

impl Default for ContextOptions {
    fn default() -> Self {
        ContextOptions {
            path: None,
            snapshot_lfu_capacity: 128,
        }
    }
}

impl ContextOptions {
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn snapshot_lfu_capacity(mut self, capacity: usize) -> Self {
        self.snapshot_lfu_capacity = capacity;
        self
    }
}
