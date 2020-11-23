use crate::commit::CommitIdentifier;
use crate::db::{data as D, DB};
use crate::snapshot::Snapshot;
use crossbeam::sync::{ShardedLock, ShardedLockReadGuard, ShardedLockWriteGuard};
use lfu::LFUCache;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Context {
    db: Arc<ShardedLock<DB>>,
    snapshot_cache: Mutex<LFUCache<CommitIdentifier, Snapshot>>,
}

impl Context {
    pub fn new(options: &ContextOptions) -> Self {
        let path = options.path.clone().unwrap();
        Context {
            db: Arc::new(ShardedLock::new(DB::open(&path))),
            snapshot_cache: Mutex::new(
                LFUCache::with_capacity(options.snapshot_lfu_capacity).unwrap(),
            ),
        }
    }

    fn db_write(&self) -> Result<ShardedLockWriteGuard<DB>, ContextError> {
        self.db.write().map_err(|_| ContextError::AcquireWriteLock)
    }

    fn db_read(&self) -> Result<ShardedLockReadGuard<DB>, ContextError> {
        self.db.read().map_err(|_| ContextError::AcquireReadLock)
    }

    /// Return the snapshot of a commit.
    pub fn snapshot(&mut self, commit: CommitIdentifier) -> Result<Snapshot, ContextError> {
        let mut cache = self
            .snapshot_cache
            .lock()
            .map_err(|_| ContextError::AcquireLock)?;

        if let Some(snapshot) = cache.get(&commit) {
            return Ok(snapshot.clone());
        }

        let snapshot = {
            let db = self.db_read()?;
            match db
                .get(D::SnapshotKey(commit.clone()))
                .map_err(|e| ContextError::DBError(e))?
            {
                Some(snapshot) => snapshot,
                None => Snapshot::default(),
            }
        };

        cache.set(commit, snapshot.clone());
        Ok(snapshot)
    }
}

#[derive(Debug)]
pub enum ContextError {
    DBError(rocksdb::Error),
    AcquireWriteLock,
    AcquireReadLock,
    AcquireLock,
}

impl std::error::Error for ContextError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            ContextError::DBError(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::DBError(e) => write!(f, "RocksDB error: {}", e),
            ContextError::AcquireWriteLock => write!(f, "Could not acquire the write lock to DB."),
            ContextError::AcquireReadLock => write!(f, "Could not acquire the read lock to DB."),
            ContextError::AcquireLock => write!(f, "Could not acquire a lock."),
        }
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
