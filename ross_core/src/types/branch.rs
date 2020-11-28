use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// The unique identifier of a branch.
pub type BranchId = Hash16;

/// Branch identifier is used to limit the key space of branches in
/// a repository.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct BranchIdentifier {
    pub repository: RepositoryId,
    pub uuid: BranchId,
}

/// The point in which this branch was forked form.
/// ```text
/// - C0 - C1 - C2 ---------> Main
///         \_______ C3 ----> Forked
/// ```
/// For example in the above case the `fork_point` of branch `Forked` is
/// `(Branch(Main), Commit(C1))`.
///
/// TODO(qti3e) Consider investigating this edge-case.
/// ```text
/// - C0 - C1 - C2 -------------> Main
///         \_______ C3 -------> Forked
///         \____________C4 ---> Fork 2
/// `Fork 2` is forked from `Forked` and not `Main`.
/// LCA(C4, C3) = LCA(C4, C2) = C1
/// ```
pub type ForkPoint = Option<(BranchIdentifier, CommitIdentifier)>;

/// The information regarding a branch, this struct is stored in the DB.
#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Current head of the commit.
    pub head: Option<CommitIdentifier>,
    /// If `head` is the last commit on a branch, `fork_point` is the tail.  
    /// See [ForkPoint](ForkPoint).
    pub fork_point: ForkPoint,
    /// When was this branch created.
    pub created_at: Timestamp,
    /// The user who created the branch.
    pub user: UserId,
    /// An static branch is a branch that cannot have live-changes and can therefore
    /// only be updated using merges.  
    /// As a usage we can point to `production` branch in a repository.
    pub is_static: bool,
    /// An archived branch is a read-only branch, no further changes are allowed.
    pub is_archived: bool,
    /// The human-readable title of the branch for display-only purposes.
    pub title: String,
}
