use crate::db_schema;
use crate::types::*;
use serde::{Deserialize, Serialize};

db_schema!((DbKey, DbWriteKey, DbReadKey) {
    /// Store the history of a repository.
    cf LOG(Log:RepositoryId) -> Vec<LogEvent> {},
    /// Store information regarding each repository.
    cf REPOSITORIES(Repository:RepositoryId) -> RepositoryInfo {
        /// Used to check if a repository exists without the cost of deserializing
        /// it's information.
        RepositoryExists -> ();
    },
    /// Store information regarding each branch.
    cf BRANCHES(Branch:BranchIdentifier) -> BranchInfo {},
    /// Column family used to store merge-branches.
    cf MERGE_BRANCHES(MergeBranch:MergeBranchId) -> MergeBranchInfo {
        /// Used to check if a merge branch exists, without the deserializing cost.
        MergeBranchExists -> ();
    },
    /// Map each commit identifier to the commit data.
    cf COMMITS(Commit:CommitIdentifier) -> CommitInfo {
        CommitOrigin -> CommitInfoOrigin;
    },
    /// Store a vector of live changes that are not yet committed.
    cf LIVE_CHANGES(LiveChanges:BranchOrMergeBranch) -> Vec<Patch> {},
    /// We only store a limited number of patches in LIVE_CHANGES, after a
    /// threshold we compute the delta of a branch/merge-branch to its original
    /// state and store that instead of all other patches.
    cf PACKED_DELTA(PackedDelta:BranchOrMergeBranch) -> Delta {},
    /// This column family is used to store the snapshot of each commit.
    cf SNAPSHOT(CommitSnapshot:CommitIdentifier) -> SnapshotEntry {}
});

#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum BranchOrMergeBranch {
    Branch(BranchIdentifier),
    MergeBranch(MergeBranchId),
}

impl From<BranchIdentifier> for BranchOrMergeBranch {
    fn from(id: BranchIdentifier) -> Self {
        BranchOrMergeBranch::Branch(id)
    }
}

impl From<MergeBranchId> for BranchOrMergeBranch {
    fn from(id: MergeBranchId) -> Self {
        BranchOrMergeBranch::MergeBranch(id)
    }
}
