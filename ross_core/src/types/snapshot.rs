use super::{CommitIdentifier, Delta, State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SnapshotEntry {
    Delta {
        base: CommitIdentifier,
        delta: Delta,
    },
    Snapshot(State),
}
