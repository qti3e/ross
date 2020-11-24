use crate::hash::{Hash16, Hash20};
use crate::{Timestamp, UserID};
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
        hash: Hash20,
        branch: Hash16,
    },
    Merge {
        time: Timestamp,
        uid: UserID,
        to: Hash16,
        from: Vec<Hash16>,
    },
    BranchCreated {
        time: Timestamp,
        uid: UserID,
        uuid: Hash16,
        name: String,
        head: Hash20,
    },
    BranchDeleted {
        time: Timestamp,
        uid: UserID,
        uuid: Hash16,
        head: Hash20,
    },
}
