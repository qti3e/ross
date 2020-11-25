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
    info: branch::BranchInfo,
    db: DBSync,
    live_changes: Vec<action::Transaction>,
    snapshot: snapshot::Snapshot,
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
        if self.info.is_static {
            Err(res::Error::WriteOnStatic)
        } else if self.info.is_archived {
            Err(res::Error::WriteOnArchived)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn put_info(&self, batch: &mut Batch) {
        batch.put(keys::BranchInfo(self.id), &self.info);
    }

    /// Try to perform the given transaction on this session.
    pub fn perform(&mut self, trx: action::Transaction) -> res::Result<()> {
        self.check_write()?;
        if trx.actions.len() == 0 {
            return Err(res::Error::EmptyTransaction);
        }

        let snapshot = self
            .snapshot
            .perform(&trx.actions)
            .map_err(|c| res::Error::Conflict(c.iter().map(|x| x.into()).collect()))?;
        // New snapshot is created and it did not have any conflicts.
        let mut batch = Batch::new();
        batch.push(keys::LiveChanges(self.id.clone()), &trx);
        self.db_perform(batch)?;
        // Update the current instance.
        self.snapshot = snapshot;
        self.live_changes.push(trx);
        Ok(())
    }

    /// Commit the active changes.
    pub fn commit(
        &mut self,
        committer: UserID,
        message: String,
    ) -> res::Result<commit::CommitIdentifier> {
        self.check_write()?;
        if self.live_changes.len() == 0 {
            return Err(res::Error::NoChangeToCommit);
        }

        let mut commit = commit::CommitInfo {
            branch: self.id,
            fork_point: self.info.fork_point,
            parents: vec![self.info.head],
            time: now(),
            committer,
            message,
            actions: Vec::new(),
        };

        // Move the live changes to the commit, we must restore them if the
        // the operation fails for any reason.
        std::mem::swap(&mut commit.actions, &mut self.live_changes);

        let mut batch = Batch::new();
        // Write the commit.
        let commit_id = commit.commit(&mut batch, self.id.repository, &self.snapshot);
        // Change the branch's head.
        let old_head = std::mem::replace(&mut self.info.head, commit_id);
        self.put_info(&mut batch);
        // Clear the live changes.
        batch.delete(keys::LiveChanges(self.id));

        if let Err(e) = self.db_perform(batch) {
            std::mem::swap(&mut commit.actions, &mut self.live_changes);
            self.info.head = old_head;
            return Err(e);
        }

        Ok(commit_id)
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
        EmptyTransaction,
        NoChangeToCommit,
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
                Error::EmptyTransaction => {
                    write!(f, "Transaction must contain at lease one action.")
                }
                Error::NoChangeToCommit => write!(f, "No changes added to commit."),
            }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}
