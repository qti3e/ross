use crate::db::{keys, DBSync};
use crate::prelude::*;
use crate::sync;
use serde::{Deserialize, Serialize};

sync!(EditorSync(Editor) {
  /// The id of the branch.
  id: BranchIdentifier,
  /// Reference to the context, we store it so that we can inform context
  /// when the editor needs to be dropped.
  ctx: ContextSync,
  /// The user who is listening to this editor.
  user: Option<UserId>
});

impl Drop for EditorSync {
    fn drop(&mut self) {
        let rc = std::sync::Arc::strong_count(&self.inner);
        // Why 2? When this method is called our content is not dropped yet, so
        // we are still keeping a reference to the inner Arc (1 ref), we also always
        // have a reference to the editor in the context, which is another ref.
        // So when we reach 2 references, it basically means no one is actually
        // listening to this editor anymore and we are free to drop it from the map.
        if rc == 2 {
            if let Ok(mut ctx) = self.ctx.write() {
                ctx.drop_editor(self.id);
            }
        }
    }
}

impl EditorSync {
    pub fn open(&self, user: UserId) -> Self {
        let mut cloned = self.clone();
        cloned.user = Some(user);
        cloned
    }
}

/// A real-time editor on top of a branch, it should be guarded with `EditorSync`.
pub struct Editor {
    db: DBSync,
    snapshot: Snapshot,
    id: BranchIdentifier,
    live_changes: Vec<BatchPatch>,
    head: CommitHash,
}

impl Editor {
    /// Called by a synced client to perform a transaction.
    pub fn perform(&mut self, batch: BatchPatch) -> Result<EditorResponse> {
        let revert_patch = match self.snapshot.apply_batch_patch(&batch.patches, false) {
            Ok(revert_patch) => revert_patch,
            Err(conflicts) => {
                return Ok(EditorResponse {
                    others: None,
                    current: Some(EditorMessage::Conflicts(conflicts)),
                })
            }
        };

        let do_write = || -> Result<()> {
            let mut db = self.db.write()?;
            db.push(keys::LiveChanges(self.id), &batch)?;
            Ok(())
        };

        match do_write() {
            Ok(()) => {
                self.live_changes.push(batch);
                Ok(EditorResponse {
                    others: Some(EditorMessage::Patch(self.live_changes.last().unwrap())),
                    current: None,
                })
            }
            Err(error) => {
                self.snapshot
                    .apply_batch_patch(&revert_patch, true)
                    .unwrap();
                Err(error)
            }
        }
    }

    /// Performs the initial sync.
    pub fn sync(&self) -> EditorMessage {
        EditorMessage::FullSync {
            head: SessionHead {
                commit: self.head,
                live: self.live_changes.len(),
            },
            snapshot: &self.snapshot,
        }
    }

    /// A partial sync happens when reconnecting to the server after a period in which
    /// the user was offline.
    pub fn partial_sync(
        &mut self,
        head: SessionHead,
        batches: Vec<BatchPatch>,
    ) -> Result<EditorResponse> {
        let same_commit = self.head == head.commit;
        let same_live = self.live_changes.len() == head.live;

        match (same_commit, same_live, batches.len()) {
            (true, true, 0) => {
                // There were no activities both on the server and the client while
                // the user was offline. -> Don't do anything.
                Ok(EditorResponse {
                    others: None,
                    current: None,
                })
            }
            (true, true, _) => {
                // There were no activities on the server while the user was offline,
                // but there were changes made by the user in that period.
                // -> Sync the server and other users.
                unimplemented!();
            }
            (true, false, 0) => {
                // There were no new commits, but there were some changes on the server,
                // but nothing on the user's side.
                // -> We only need to sync the current client.
                unimplemented!();
            }
            (true, false, _) => {
                // There were no new commits, but there were some changes on both server,
                // and the client.
                // -> Sync the current client & try to apply new changes made by the user
                // and sync everyone else.
                unimplemented!();
            }
            (false, _, 0) => {
                // There was a commit, but no new changes were made by the user.
                // So just send a full-sync response, the user will figure out the delta.
                Ok(EditorResponse {
                    others: None,
                    current: Some(self.sync()),
                })
            }
            (false, _, _) => {
                // There was a commit, and new changes were made by the user.
                // So send a full-sync response, and apply patches by the user and
                // update everyone else too.
                unimplemented!();
            }
        }
    }

    /// Commit the live changes, it will only commit the changes if the head given
    /// in parameters is still valid.
    /// We may introduce `lock` and `unlock` to lock an editor temporarily before
    /// sending the commit request.
    pub fn commit(
        &mut self,
        user: UserId,
        head: SessionHead,
        message: EditorMessage,
    ) -> Result<EditorResponse> {
        unimplemented!()
    }
}

/// The state that a client is in a editor.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHead {
    /// Hash of the latest commit in the session.
    commit: CommitHash,
    /// Index of the last transaction after the head commit.
    live: usize,
}

/// A message sent from the server to the client.
#[derive(Debug, Serialize)]
pub enum EditorMessage<'a> {
    /// The perform request had conflicts and therefore was not applied.
    Conflicts(Vec<PatchConflict>),
    /// A patch that needs to applied on the client side.
    Patch(&'a BatchPatch),
    /// An snapshot of the entire state of the session.
    FullSync {
        head: SessionHead,
        snapshot: &'a Snapshot,
    },
}

/// Result of an action on the session which is two optional messages,
/// one of which needs to be broadcasted to everyone except the current
/// user and the other needs to be sent to the user who initiated the
/// request.
#[derive(Debug, Serialize)]
pub struct EditorResponse<'a> {
    pub others: Option<EditorMessage<'a>>,
    pub current: Option<EditorMessage<'a>>,
}
