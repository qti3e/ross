use crate::commit::CommitIdentifier;
use crate::db::DB;
use crossbeam::sync::ShardedLock;
use lfu::LFUCache;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Context {
    /// The database instance that is used as our backend data-storage.
    db: Arc<ShardedLock<DB>>,
}

impl Context {
    pub fn new(options: &ContextOptions) -> Self {
        unimplemented!()
    }

    /// Remove all of the closed branches from the cache.
    pub fn evict(&mut self) {}
}

pub struct ContextOptions {
    path: String,
    checkout_lfu_capacity: usize,
}

impl Default for ContextOptions {
    fn default() -> Self {
        ContextOptions {
            path: "/tmp/ross-db".to_string(),
            checkout_lfu_capacity: 128,
        }
    }
}

impl ContextOptions {
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    pub fn checkout_lfu_capacity(mut self, capacity: usize) -> Self {
        self.checkout_lfu_capacity = capacity;
        self
    }
}
