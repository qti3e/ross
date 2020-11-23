use rocksdb;

pub struct DB {
    db: rocksdb::DB,
}

impl DB {
    /// Open a new database instance.
    pub fn open(path: &str) -> Self {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        // opts.set_merge_operator("bincode-vec-append", full_merge_fn, None);
        let db = rocksdb::DB::open(&opts, path).unwrap();
        DB { db }
    }

    /// Perform the given transaction on the database.
    pub fn perform(&self, batch: Batch) -> bool {
        self.db.write(batch.finalize()).is_ok()
    }
}

pub struct Batch {
    batch: rocksdb::WriteBatch,
}

impl Batch {
    pub fn new() -> Self {
        Batch {
            batch: rocksdb::WriteBatch::default(),
        }
    }

    pub fn put<K, V: serde::Serialize>(&mut self, key: K, value: V)
    where
        K: data::DBKey<V>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let value = bincode::serialize(&value).unwrap();
        self.batch.put(key, value);
    }

    pub fn delete<K, V>(&mut self, key: K)
    where
        K: data::DBKey<V>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        self.batch.delete(key);
    }

    pub fn delete_range<K, V>(&mut self, from: K, to: K)
    where
        K: data::DBKey<V>,
    {
        let from = bincode::serialize(&from.key()).unwrap();
        let to = bincode::serialize(&to.key()).unwrap();
        self.batch.delete_range(from, to);
    }

    pub fn append<K, V, I: serde::Serialize>(&mut self, key: K, item: I)
    where
        K: data::DBKey<V> + data::DBKeyVec<I>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let item = bincode::serialize(&item).unwrap();
        self.batch.merge(key, item);
    }

    pub fn finalize(self) -> rocksdb::WriteBatch {
        self.batch
    }
}

/// This module contains all of the keys and values that can be used in
/// our rocksdb instance.
/// We have these key groups:
/// 1. Branches: Project(uuid) -> Vec<BranchUUID>
/// 2. BranchInfo: Branch(uuid) -> BranchInfo
/// 3. LiveChange: Branch(uuid) -> Vec<Transaction>
/// 4. CommitInfo: Commit(hash) -> CommitInfo
/// 5. Snapshot: Commit(hash) -> Snapshot
/// 6. Log: Project(uuid) -> Vec<Log>
pub mod data {
    use crate::action::Transaction;
    use crate::branch::{BranchIdentifier, BranchInfo};
    use crate::commit::{CommitIdentifier, CommitInfo};
    use crate::hash::Hash16;
    use crate::log::LogItem;
    use serde::{Deserialize, Serialize};

    pub trait DBKey<Value> {
        fn key(self) -> Key;
    }

    pub trait DBKeyVec<Item> {}

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Key {
        Branches(BranchesKey),
        BranchInfo(BranchInfoKey),
        LiveChanges(LiveChangesKey),
        CommitInfo(CommitInfoKey),
        Snapshot(SnapshotKey),
        Log(LogKey),
    }

    // ---- Branches

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchesKey(pub Hash16);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchesValue(pub Vec<Hash16>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchesAppendItem(pub Hash16);

    impl DBKey<BranchesValue> for BranchesKey {
        fn key(self) -> Key {
            Key::Branches(self)
        }
    }

    impl DBKeyVec<BranchesAppendItem> for BranchesKey {}

    // ---- BranchInfo

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchInfoKey(pub BranchIdentifier);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchInfoValue(pub BranchInfo);

    impl DBKey<BranchInfoValue> for BranchInfoKey {
        fn key(self) -> Key {
            Key::BranchInfo(self)
        }
    }

    // ---- LiveChanges

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesKey(pub BranchIdentifier);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesValue(pub Vec<Transaction>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesAppendItem(pub Transaction);

    impl DBKey<LiveChangesValue> for LiveChangesKey {
        fn key(self) -> Key {
            Key::LiveChanges(self)
        }
    }

    impl DBKeyVec<LiveChangesAppendItem> for LiveChangesKey {}

    // ---- CommitInfo

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommitInfoKey(pub CommitIdentifier);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommitInfoValue(pub CommitInfo);

    impl DBKey<CommitInfoValue> for CommitInfoKey {
        fn key(self) -> Key {
            Key::CommitInfo(self)
        }
    }

    // ---- Snapshot

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SnapshotKey(pub CommitIdentifier);

    // ---- Log

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LogKey(pub Hash16);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LogValue(pub Vec<LogItem>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LogAppendItem(pub LogItem);

    impl DBKey<LogValue> for LogKey {
        fn key(self) -> Key {
            Key::Log(self)
        }
    }

    impl DBKeyVec<LogAppendItem> for LogKey {}
}
