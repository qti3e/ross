use crate::db::DBSync;
use crate::prelude::*;
use crate::sync;
use serde::{Deserialize, Serialize};

sync!(SessionSync(Session) {
  /// The id of the branch.
  id: BranchIdentifier,
  /// Reference to the context, we store it so that we can inform context
  /// when the session needs to be dropped.
  ctx: ContextSync,
  /// The user who is listening to this session.
  user: Option<UserId>
});

impl Drop for SessionSync {
    fn drop(&mut self) {
        let rc = std::sync::Arc::strong_count(&self.inner);
        // Why 2? When this method is called our content is not dropped yet, so
        // we are still keeping a reference to the inner Arc (1 ref), we also always
        // have a reference to the session in the context, which is another ref.
        // So when we reach 2 references, it basically means no one is actually
        // listening to this session anymore and we are free to drop it from the map.
        if rc == 2 {
            if let Ok(mut ctx) = self.ctx.write() {
                ctx.drop_session(self.id);
            }
        }
    }
}

impl SessionSync {
    pub fn open(&self, user: UserId) -> Self {
        let mut cloned = self.clone();
        cloned.user = Some(user);
        cloned
    }
}

pub struct Session {
    db: DBSync,
    snapshot: Snapshot,
    branch: BranchInfo,
    live_changes: Vec<BatchPatch>,
}

impl Session {
    /// Called by a synced client to perform a transaction, the response
    /// is a result, if `Ok`, the value needs to be broadcasted to other
    /// users, the `Err` however needs to be sent to the client who
    /// initiated the request.
    pub fn perform(&mut self, batch: BatchPatch) {
        match self.snapshot.apply_batch_patch(&batch.patches, false) {
            Ok(revert_patch) => {}
            Err(conflicts) => {}
        }
    }

    pub fn sync(&self) {}

    pub fn partial_sync(&mut self, head: SessionHead, batches: Vec<BatchPatch>) {}
}

/// Head of the session.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHead {
    /// Hash of the latest commit in the session.
    commit: CommitHash,
    /// Index of the last transaction after the head commit.
    live: usize,
}
