use crate::db::DB;
use crossbeam::sync::ShardedLock;
use lfu::LFUCache;
use std::sync::Arc;

pub struct Context {
    db: Arc<ShardedLock<DB>>,
}

impl Context {
    pub fn new(options: &ContextOptions) -> Self {
        let path = options.path.clone().unwrap();
        Context {
            db: Arc::new(ShardedLock::new(DB::open(&path))),
        }
    }
}

pub struct ContextOptions {
    pub path: Option<String>,
    pub checkout_lfu_capacity: usize,
}

impl Default for ContextOptions {
    fn default() -> Self {
        ContextOptions {
            path: None,
            checkout_lfu_capacity: 128,
        }
    }
}

impl ContextOptions {
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn checkout_lfu_capacity(mut self, capacity: usize) -> Self {
        self.checkout_lfu_capacity = capacity;
        self
    }
}
