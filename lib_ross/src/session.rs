use crate::action::Transaction;
use crate::branch::BranchIdentifier;
use crate::conflict::Conflict;
use crate::db::DBSync;
use crate::error::Result;
use crate::snapshot::Snapshot;
use crate::sync;

sync!(sync SessionSync(Session) {});

pub struct Session {
    id: BranchIdentifier,
    db: DBSync,
    live_changes: Vec<Transaction>,
    snapshot: Snapshot,
    is_archived: bool,
    is_static: bool,
}

impl Session {
    pub fn perform(&mut self, action: Transaction) -> Result<Option<Vec<Conflict>>> {
        let snapshot = self.snapshot.perform(&action.actions);
        Ok(None)
    }
}

pub mod response {}
