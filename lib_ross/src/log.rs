use crate::{BranchID, CommitID, Timestamp, UserID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogItem {
    Init {
        time: Timestamp,
        uid: UserID,
    },
    Commit {
        time: Timestamp,
        uid: UserID,
        hash: CommitID,
        branch: BranchID,
    },
    Merge {
        time: Timestamp,
        uid: UserID,
        to: BranchID,
        from: Vec<CommitID>,
    },
    BranchCreated {
        time: Timestamp,
        uid: UserID,
        uuid: BranchID,
        name: String,
        head: CommitID,
    },
    BranchDeleted {
        time: Timestamp,
        uid: UserID,
        uuid: BranchID,
        head: CommitID,
    },
}
