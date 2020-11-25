use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct BranchIdentifier {
    pub repository: RepositoryID,
    pub uuid: BranchID,
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
    pub head: commit::CommitIdentifier,
    pub fork_point: Option<(BranchIdentifier, commit::CommitIdentifier)>,
    pub name: String,
    pub created_at: Timestamp,
    pub user: UserID,
    pub is_static: bool,
    pub is_archived: bool,
}

impl BranchInfo {
    pub fn status(&self) -> BranchStatus {
        match (self.is_static, self.is_archived) {
            (false, false) => BranchStatus::Normal,
            (true, false) => BranchStatus::Static,
            (false, true) => BranchStatus::Archived,
            _ => BranchStatus::StaticArchived,
        }
    }
}
