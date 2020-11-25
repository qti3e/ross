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
        opts.set_merge_operator("bincode-vec-append", vec_push_merge, None);
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
    pub fn push<K, I: serde::Serialize>(&mut self, key: K, item: &I)
    where
        K: keys::DBKey<Vec<I>>,
    {
        let key = bincode::serialize(&key.key()).unwrap();
        let item = bincode::serialize(item).unwrap();
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
    use crate::*;
    use serde::{Deserialize, Serialize};

    db_keys!(DBKey(Key) {
        /// Log all of the events in a repository since its creation.
        Log(RepositoryID) -> Vec<log::LogEvent>,
        /// List of all the branches that a repository owns.
        Branches(RepositoryID) -> Vec<BranchID>,
        /// Store the information regarding each branch.
        BranchInfo(branch::BranchIdentifier) -> branch::BranchInfo,
        /// Each non-static branch can have list of pending actions which
        /// are not committed yet.
        LiveChanges(branch::BranchIdentifier) -> Vec<action::Transaction>,
        /// This key-group is used to store commits.
        CommitInfo(commit::CommitIdentifier) -> commit::CommitInfo,
        /// Store a snapshot of the whole object set for each commit.
        Snapshot(commit::CommitIdentifier) -> snapshot::Snapshot
    });
}

#[inline]
fn vec_push_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &mut rocksdb::MergeOperands,
) -> Option<Vec<u8>> {
    let result = merge(existing_val, operands);
    Some(result)
}
