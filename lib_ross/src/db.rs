use crate::bincode_vec_append::merge;
use crate::error::{Error, Result};
use crate::sync;
use rocksdb;

sync!(sync DBSync(DB) {});

/// A typespace layer on the top of rocksdb.
pub struct DB {
    db: rocksdb::DB,
}

impl DB {
    /// Open a new database instance.
    pub fn open(path: &str) -> Self {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_merge_operator("bincode-vec-append", append_merge, None);
        let db = rocksdb::DB::open(&opts, path).unwrap();
        DB { db }
    }

    /// Perform the given transaction on the database.
    #[inline(always)]
    pub fn perform(&mut self, batch: Batch) -> Result<()> {
        self.db
            .write(batch.finalize())
            .map_err(|e| Error::DBError(e))
    }

    /// Return the data associated with the given key.
    pub fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: keys::DBKey<V>,
        V: serde::de::DeserializeOwned,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let pinned = self.db.get_pinned(key).map_err(|e| Error::DBError(e))?;
        let bytes = match pinned {
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

    #[inline(always)]
    pub fn put<K, V: serde::Serialize>(&mut self, key: K, value: &V)
    where
        K: keys::DBKey<V>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let value = bincode::serialize(value).unwrap();
        self.batch.put(key, value);
    }

    #[inline(always)]
    pub fn delete<K, V>(&mut self, key: K)
    where
        K: keys::DBKey<V>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        self.batch.delete(key);
    }

    #[inline(always)]
    pub fn delete_range<K, V>(&mut self, from: K, to: K)
    where
        K: keys::DBKey<V>,
    {
        let from = bincode::serialize(&from.key()).unwrap();
        let to = bincode::serialize(&to.key()).unwrap();
        self.batch.delete_range(from, to);
    }

    #[inline(always)]
    pub fn append<K, I: serde::Serialize>(&mut self, key: K, item: I)
    where
        K: keys::DBKey<Vec<I>>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let item = bincode::serialize(&item).unwrap();
        self.batch.merge(key, item);
    }

    #[inline(always)]
    pub(crate) fn finalize(self) -> rocksdb::WriteBatch {
        self.batch
    }
}

/// This module contains all of the keys that can be used in
/// our rocksdb instance.
/// We have these key groups:
/// 1. Branches: Repository(uuid) -> Vec<BranchUUID>
/// 2. BranchInfo: Branch(uuid) -> BranchInfo
/// 3. LiveChange: Branch(uuid) -> Vec<Transaction>
/// 4. CommitInfo: Commit(hash) -> CommitInfo
/// 5. Snapshot: Commit(hash) -> Snapshot
/// 6. Log: Repository(uuid) -> Vec<Log>
pub mod keys {
    use crate::action::Transaction;
    use crate::branch::{BranchIdentifier, BranchInfo};
    use crate::commit::{CommitIdentifier, CommitInfo};
    use crate::hash::{Hash16, Hash20};
    use crate::log::LogItem;
    use crate::snapshot::Snapshot;
    use serde::{Deserialize, Serialize};

    pub trait DBKey<Value> {
        fn key(self) -> Key;
    }

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

    impl DBKey<Vec<Hash16>> for BranchesKey {
        fn key(self) -> Key {
            Key::Branches(self)
        }
    }

    // ---- BranchInfo

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchInfoKey(pub BranchIdentifier);

    impl DBKey<BranchInfo> for BranchInfoKey {
        fn key(self) -> Key {
            Key::BranchInfo(self)
        }
    }

    impl BranchInfoKey {
        pub fn all(repository: Hash16) -> (Self, Self) {
            let min = BranchIdentifier {
                repository,
                uuid: Hash16::MIN,
            };
            let max = BranchIdentifier {
                repository,
                uuid: Hash16::MAX,
            };
            (Self(min), Self(max))
        }
    }

    // ---- LiveChanges

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesKey(pub BranchIdentifier);

    impl DBKey<Vec<Transaction>> for LiveChangesKey {
        fn key(self) -> Key {
            Key::LiveChanges(self)
        }
    }

    impl LiveChangesKey {
        pub fn all(repository: Hash16) -> (Self, Self) {
            let min = BranchIdentifier {
                repository,
                uuid: Hash16::MIN,
            };
            let max = BranchIdentifier {
                repository,
                uuid: Hash16::MAX,
            };
            (Self(min), Self(max))
        }
    }

    // ---- CommitInfo

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommitInfoKey(pub CommitIdentifier);

    impl DBKey<CommitInfo> for CommitInfoKey {
        fn key(self) -> Key {
            Key::CommitInfo(self)
        }
    }

    impl CommitInfoKey {
        pub fn all(repository: Hash16) -> (Self, Self) {
            let min = CommitIdentifier {
                repository,
                hash: Hash20::MIN,
            };
            let max = CommitIdentifier {
                repository,
                hash: Hash20::MAX,
            };
            (Self(min), Self(max))
        }
    }

    // ---- Snapshot

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SnapshotKey(pub CommitIdentifier);

    impl DBKey<Snapshot> for SnapshotKey {
        fn key(self) -> Key {
            Key::Snapshot(self)
        }
    }

    impl SnapshotKey {
        pub fn all(repository: Hash16) -> (Self, Self) {
            let min = CommitIdentifier {
                repository,
                hash: Hash20::MIN,
            };
            let max = CommitIdentifier {
                repository,
                hash: Hash20::MAX,
            };
            (Self(min), Self(max))
        }
    }

    // ---- Log

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LogKey(pub Hash16);

    impl DBKey<Vec<LogItem>> for LogKey {
        fn key(self) -> Key {
            Key::Log(self)
        }
    }
}

#[inline]
fn append_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &mut rocksdb::MergeOperands,
) -> Option<Vec<u8>> {
    let result = merge(existing_val, operands);
    Some(result)
}
