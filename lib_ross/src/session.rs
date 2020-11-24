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

/// A session is the gateway to interact with a branch, each branch can have
/// one and only one active session at a time, to edit the same branch from
/// multiple parties you should consider using `SessionSync`, one should never
/// construct a session on their own and should only rely on `Context` to return
/// a `SessionSync`.
pub struct Session {
    id: branch::BranchIdentifier,
    db: DBSync,
    live_changes: Vec<action::Transaction>,
    snapshot: snapshot::Snapshot,
    is_archived: bool,
    is_static: bool,
}

impl Session {
    #[inline]
    fn db_perform(&mut self, batch: Batch) -> res::Result<()> {
        self.db
            .write()
            .map_err(|e| res::Error::Internal(e))?
            .perform(batch)
            .map_err(|e| res::Error::Internal(e))
    }

    #[inline]
    fn check_write(&self) -> res::Result<()> {
        if self.is_static {
            Err(res::Error::WriteOnStatic)
        } else if self.is_archived {
            Err(res::Error::WriteOnArchived)
        } else {
            Ok(())
        }
    }

    /// Try to perform the given transaction on this session.
    pub fn perform(&mut self, trx: action::Transaction) -> res::Result<()> {
        self.check_write()?;
        let snapshot = self
            .snapshot
            .perform(&trx.actions)
            .map_err(|c| res::Error::Conflict(c.iter().map(|x| x.into()).collect()))?;
        // New snapshot is created and it did not have any conflicts.
        let mut batch = Batch::new();
        batch.push(keys::LiveChangesKey(self.id.clone()), &trx);
        self.db_perform(batch)?;
        // Update the current instance.
        self.snapshot = snapshot;
        self.live_changes.push(trx);
        Ok(())
    }
}

pub mod res {
    use serde::Serialize;
    use std::error;
    use std::fmt;

    #[derive(Debug, Serialize)]
    #[serde(tag = "type", content = "body", rename_all = "camelCase")]
    pub enum Error {
        Internal(#[serde(skip_serializing)] crate::error::Error),
        Conflict(Vec<crate::conflict::JsonConflict>),
        WriteOnArchived,
        WriteOnStatic,
    }

    impl error::Error for Error {
        fn cause(&self) -> Option<&dyn error::Error> {
            match self {
                Error::Internal(e) => Some(e),
                _ => None,
            }
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::Internal(e) => write!(f, "Internal error: {}", e),
                Error::Conflict(_) => write!(f, "Action yields conflict."),
                Error::WriteOnArchived => write!(f, "Cannot write on an archived branch."),
                Error::WriteOnStatic => write!(f, "Cannot write on an static branch."),
            }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}
