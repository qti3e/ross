use crate::db_keys;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

// TODO(qti3e) We must utilize column families, I have feeling if used correctly
// it can improve the performance dramatically.

db_keys!(DBKey(Key) {
  /// Log all the actions happening in a repository.
  Log(RepositoryId) -> Vec<LogEvent>,
  /// List of all the branches owned by the repository.
  Branches(RepositoryId) -> Vec<BranchId>,
  /// Store/retrieve the information about a branch.
  Branch(BranchIdentifier) -> BranchInfo,
  /// Store the information regarding each commit.
  Commit(CommitIdentifier) -> CommitInfo,
  /// Delta of the commit relative to the parent.
  CommitDelta(CommitIdentifier) -> CompactDelta,
  /// Yes! we are indeed storing the entire state of each commit.
  // TODO(qti3e) Use techniques to reduce data usage in snapshots:
  // Possible candidates:
  // 1. Store the diff relative to the last snapshot.
  //   If we have three commits `A -> B -> C` and we are inserting a new commit
  //   `C -> D`, get the last snapshot of C, run a diff, if the diff size (number
  //   of action that we need to do in order to apply the diff) is less than a
  //   the number provided by option `SnapshotCompactMinDiff` (default = 128)
  //   store a DiffSnapshot, that points to an actual snapshot (not another
  //   DiffSnapshot) and the CompactDiff that needs to applied.
  // 2. After a while, store a PackedSnapshot, which looks like this:
  //   objects = Map<(ObjectId, ObjectVersion), ObjectData>
  //   states = Map<CommitHash, Vec<(ObjectId, ObjectVersion)>>
  //   Compress(Pack { objects, states })
  CommitSnapshot(CommitIdentifier) -> Snapshot
});

db_partial_keys!(DBKey(Key)::PartialDBKey {
  /// Get the number of branches owned by a repository.
  Branches(RepositoryId)::NumberOfBranches -> u64,
  /// Head of a branch.
  Branch(BranchIdentifier)::BranchHead -> CommitIdentifier,
  /// Get the frequently accessed origin info of a commit.
  Commit(CommitIdentifier)::CommitOrigin -> CommitOrigin
});
