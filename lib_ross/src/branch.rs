use crate::hash::{Hash16, Hash20};
use crate::{Timestamp, UserID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct BranchIdentifier {
    pub repository: Hash16,
    pub uuid: Hash16,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BranchStatus {
    /// This is a normal branch, it's writable and supports live changes.
    Normal,
    /// This branch can only be mutated by merges, no live changes are allowed.
    /// For example a `production` branch that triggers a recompilation on
    /// changes.
    Static,
    /// The branch is archived and is no longer supposed to be modified.
    Archived,
    /// Like `Archived` but the branch was static and has no live-changes list.
    StaticArchived,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    pub status: BranchStatus,
    pub head: Hash20,
    pub fork_root: Option<Hash20>,
    pub date: Timestamp,
    pub user: UserID,
    pub name: String,
}
