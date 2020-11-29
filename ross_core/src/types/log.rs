use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LogEvent {
    Init {
        user: UserId,
        time: Timestamp,
    },
    BranchCreated {
        id: BranchId,
        head: Option<CommitHash>,
        user: UserId,
        time: Timestamp,
    },
    BranchDeleted {
        id: BranchId,
        user: UserId,
        time: Timestamp,
    },
    Committed {
        branch: BranchId,
        hash: CommitHash,
        user: UserId,
        time: Timestamp,
    },
    MergeRequestCreated {
        source: BranchId,
        target: Vec<BranchId>,
        /// When merge request is created a tmp branch is created to generate preview,
        /// and resolve conflicts if any.
        merge_branch: BranchId,
        user: UserId,
        time: Timestamp,
    },
    Merged {
        source: BranchId,
        target: Vec<BranchId>,
        user: UserId,
        time: Timestamp,
    },
}
