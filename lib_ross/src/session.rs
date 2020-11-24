use crate::action::Transaction;
use crate::branch::BranchIdentifier;
use crate::conflict::Conflict;
use crate::context::ContextSync;
use crate::db::DBSync;
use crate::error::Result;
use crate::snapshot::Snapshot;
use crate::sync;

sync!(sync SessionSync(Session) {
    id: BranchIdentifier,
    ctx: ContextSync
});

impl Drop for SessionSync {
    fn drop(&mut self) {
        let count = std::sync::Arc::strong_count(&self.inner);
        if count == 2 {
            if let Ok(mut ctx) = self.ctx.write() {
                ctx.drop_session(&self.id);
            }
        }
    }
}

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
