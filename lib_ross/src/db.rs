use rocksdb;
pub use rocksdb::Error;

/// A typespace layer on the top of rocksdb.
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

    /// Perform the given transaction on the database, returns true/false indicating
    /// the success or failure of the commit.
    pub fn perform(&self, batch: Batch) -> bool {
        self.db.write(batch.finalize()).is_ok()
    }

    /// Return the data associated with the given key.
    pub fn get<K, V>(&self, key: K) -> Result<Option<V>, Error>
    where
        K: data::DBKey<V>,
        V: serde::de::DeserializeOwned,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let bytes = match self.db.get_pinned(key)? {
            Some(slice) => slice,
            None => return Ok(None),
        };
        let data = bincode::deserialize(bytes.as_ref()).unwrap();
        Ok(Some(data))
    }
}

/// An atomic batch of write operations.
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
        K: data::DBKey<V> + data::DBKeyWithAppend<I>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let item = bincode::serialize(&item).unwrap();
        self.batch.merge(key, item);
    }

    pub(crate) fn finalize(self) -> rocksdb::WriteBatch {
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
    use crate::hash::{Hash16, Hash20};
    use crate::log::LogItem;
    use serde::{Deserialize, Serialize};

    pub trait DBKey<Value> {
        fn key(self) -> Key;
    }

    pub trait DBKeyWithAppend<Item> {}

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

    impl DBKeyWithAppend<BranchesAppendItem> for BranchesKey {}

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

    impl BranchInfoKey {
        pub fn all(project: Hash16) -> (Self, Self) {
            let min = BranchIdentifier {
                project,
                uuid: Hash16::MIN,
            };
            let max = BranchIdentifier {
                project,
                uuid: Hash16::MAX,
            };
            (Self(min), Self(max))
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

    impl DBKeyWithAppend<LiveChangesAppendItem> for LiveChangesKey {}

    impl LiveChangesKey {
        pub fn all(project: Hash16) -> (Self, Self) {
            let min = BranchIdentifier {
                project,
                uuid: Hash16::MIN,
            };
            let max = BranchIdentifier {
                project,
                uuid: Hash16::MAX,
            };
            (Self(min), Self(max))
        }
    }

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

    impl CommitInfoKey {
        pub fn all(project: Hash16) -> (Self, Self) {
            let min = CommitIdentifier {
                project,
                hash: Hash20::MIN,
            };
            let max = CommitIdentifier {
                project,
                hash: Hash20::MAX,
            };
            (Self(min), Self(max))
        }
    }

    // ---- Snapshot

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SnapshotKey(pub CommitIdentifier);

    impl SnapshotKey {
        pub fn all(project: Hash16) -> (Self, Self) {
            let min = CommitIdentifier {
                project,
                hash: Hash20::MIN,
            };
            let max = CommitIdentifier {
                project,
                hash: Hash20::MAX,
            };
            (Self(min), Self(max))
        }
    }

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

    impl DBKeyWithAppend<LogAppendItem> for LogKey {}
}
