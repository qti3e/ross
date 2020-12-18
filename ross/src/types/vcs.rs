//! Types related to the VCS functionality of ROSS.
use super::{MergeConflict, Timestamp};
use crate::utils::hash::*;
use serde::{Deserialize, Serialize};

/// An opaque type that represents a User UUID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct UserId(pub Hash16);

/// An opaque type that represents a Repository UUID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct RepositoryId(pub Hash16);

/// An opaque type that represents a Branch UUID.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct BranchId(pub Hash16);

/// An opaque type that represents a Commit Hash.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct CommitHash(pub Hash20);

/// A branch id, prefixed by the repository id.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct BranchIdentifier {
    pub repository: RepositoryId,
    pub id: BranchId,
}

/// A commit hash, prefixed by the repository id.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitIdentifier {
    pub repository: RepositoryId,
    pub has: CommitHash,
}

/// There can only be at most one merge-branch between two branches at any given time.
/// It uses `BranchIdentifier` to make cross-repository merge-requests possible, also
/// it is prefixed by the target's repository id, so when inserted in the database, it
/// will be possible to get an iterator over all of the open (pending) merge requests
/// of a repository using a rocksdb prefix-iterator.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct MergeBranchId {
    pub target: BranchIdentifier,
    pub source: BranchIdentifier,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub owner: UserId,
    pub fork_of: Option<RepositoryId>,
    pub created_at: Timestamp,
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

/// The information regarding a branch.
#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Current head of the commit.
    pub head: CommitIdentifier,
    /// If `head` is the last commit on a branch, `fork_point` is the tail.  
    /// See [ForkPoint](ForkPoint).
    pub fork_point: ForkPoint,
    /// When was this branch created.
    pub created_at: Timestamp,
    /// The user who created the branch.
    pub user: UserId,
    /// A bit-v
    pub mode: BranchMode,
    /// The human-readable title of the branch for display-only purposes.
    pub title: String,
}

#[derive(Debug)]
pub enum BranchModeAtom {
    /// An static branch is a branch that cannot have live-changes and can therefore
    /// only be updated using merges.  
    /// As a usage we can point to `production` branch in a repository.
    Static = 1,
    /// An archived branch is a read-only branch, no further changes are allowed.
    Archived = 2,
    /// Only the owner can write.
    Private = 4,
    /// Only people who already have the ID can see the repository.
    Hidden = 8,
}

/// A bit-vector containing `BranchModeAtom`s.
#[derive(Debug, Serialize, Deserialize)]
pub struct BranchMode(pub u8);

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeBranchInfo {
    pub lca: CommitIdentifier,
    pub conflicts: Vec<MergeConflict>,
}

/// Information about a commit that is enough to find the LCA.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfoOrigin {
    pub branch: BranchId,
    pub fork_point: ForkPoint,
    pub order: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    // It might seem logical to use `#[serde(flatten)]` here, but it won't work and
    // panics with `bincode`.
    // https://github.com/servo/bincode/issues/245)
    pub origin: CommitInfoOrigin,
    pub time: Timestamp,
    pub parents: Vec<CommitIdentifier>,
    pub committer: UserId,
    pub authors: Vec<UserId>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[serde(untagged)]
pub enum BranchOrMergeBranchId {
    MergeBranch(MergeBranchId),
    Branch(BranchIdentifier),
}

impl From<BranchIdentifier> for BranchOrMergeBranchId {
    fn from(id: BranchIdentifier) -> Self {
        BranchOrMergeBranchId::Branch(id)
    }
}

impl From<MergeBranchId> for BranchOrMergeBranchId {
    fn from(id: MergeBranchId) -> Self {
        BranchOrMergeBranchId::MergeBranch(id)
    }
}
