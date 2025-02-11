use super::bincode::*;
use super::{iterator::*, DB};
use crate::db_schema;
use crate::types::*;

db_schema!((DbKey, DbWriteKey, DbReadKey) {
    /// Store the history of a repository.
    cf LOG(Log:RepositoryId) -> Vec<LogEvent> {},
    /// Store information regarding each repository.
    cf REPOSITORIES(Repository:RepositoryId) -> RepositoryInfo {
        /// Used to check if a repository exists without the cost of deserializing
        /// its information.
        RepositoryExists -> ();
    },
    /// Store information regarding each branch.
    cf BRANCHES(Branch:BranchIdentifier) -> BranchInfo {},
    /// Map each commit identifier to the commit data.
    cf COMMITS(Commit:CommitIdentifier) -> CommitInfo {
        CommitOrigin -> CommitInfoOrigin;
    },
    /// Store a vector of live changes that are not yet committed.
    cf LIVE_CHANGES(LiveChanges:BranchIdentifier) -> Vec<Patch> {},
    /// We only store a limited number of patches in LIVE_CHANGES, after a
    /// threshold we compute the delta of a branch/merge-branch to its original
    /// state and store that instead of all other patches.
    cf PACKED_DELTA(PackedDelta:BranchIdentifier) -> Delta {},
    /// This column family is used to store the snapshot of each commit.
    cf SNAPSHOT(CommitSnapshot:CommitIdentifier) -> SnapshotEntry {}
});
