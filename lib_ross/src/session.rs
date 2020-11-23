use crate::action::Transaction;
use crate::branch::BranchIdentifier;
use crate::db::DB;
use crate::snapshot::Snapshot;
use crossbeam::sync::ShardedLock;
use std::sync::Arc;

pub struct SharedSession(Arc<Session>);

pub struct Session {
    id: BranchIdentifier,
    db: Arc<ShardedLock<DB>>,
    live_changes: Vec<Transaction>,
    snapshot: Snapshot,
    is_archived: bool,
    is_static: bool,
}

impl Session {}

pub struct SessionError {}
