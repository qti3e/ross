use crate::db::*;
use crate::*;

sync!(sync SessionSync(Session) {
    id: branch::BranchIdentifier,
    ctx: context::ContextSync
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
    _id: branch::BranchIdentifier,
    _db: DBSync,
    _live_changes: Vec<action::Transaction>,
    _snapshot: snapshot::Snapshot,
    _is_archived: bool,
    _is_static: bool,
}

impl Session {}

pub mod response {}
