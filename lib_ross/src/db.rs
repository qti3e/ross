use rocksdb;
use crate::branch::{BranchInfo, BranchIdentifier};
use crate::action::Transaction;
use bincode::serialize;

pub struct DB {
    db: rocksdb::DB,
}

impl DB {
    pub fn open(path: &str) -> Self {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        // opts.set_merge_operator("bincode-vec-append", full_merge_fn, None);
        let db = rocksdb::DB::open(&opts, path).unwrap();
        DB {db}
    }

    pub fn append_live_change(&self, branch: BranchIdentifier, action: Transaction) -> bool {
        let key = data::Key::LiveChanges(data::LiveChangesKey(branch));
        let op = data::LiveChangesMergeOperand(action);
        let key = serialize(&key).unwrap();
        let op = serialize(&op).unwrap();
        self.db.merge(key, op).is_ok()
    }

    pub fn create_branch(&self, id: BranchIdentifier, info: BranchInfo) -> bool {
        unimplemented!()
    }

    pub fn delete_branch(&self, id: BranchIdentifier) -> bool {
        unimplemented!()
    }

    pub fn commit(&self, branch: BranchIdentifier) {
    }
}

/// This module contains all of the keys and values that can be used in
/// our rocksdb instance.
/// We have these key groups:
/// 1. Project: Project(uuid) -> Vec<BranchUUID>
/// 2. BranchInfo: Branch(uuid) -> BranchInfo
/// 3. LiveChange: Branch(uuid) -> Vec<Transaction>
/// 4. CommitInfo: Commit(hash) -> CommitInfo
/// 5. Snapshot: Commit(hash) -> Snapshot
pub mod data {
    use crate::hash::Hash16;
    use crate::branch::{BranchIdentifier, BranchInfo};
    use crate::commit::{CommitIdentifier, CommitInfo};
    use crate::action::Transaction;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Key {
        Project(ProjectKey),
        BranchInfo(BranchInfoKey),
        LiveChanges(LiveChangesKey),
        CommitInfo(CommitInfoKey),
        Snapshot(SnapshotKey),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProjectKey(pub Hash16);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProjectValue(pub Vec<Hash16>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchInfoKey(pub BranchIdentifier);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct BranchInfoValue(pub BranchInfo);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesKey(pub BranchIdentifier);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesValue(pub Vec<Transaction>);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LiveChangesMergeOperand(pub Transaction);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommitInfoKey(pub CommitIdentifier);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommitInfoValue(pub CommitInfo);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SnapshotKey(pub CommitIdentifier);
}
