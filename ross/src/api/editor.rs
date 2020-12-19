use super::Context;
use crate::db::keys;
use crate::error::*;
use crate::types::*;
use crossbeam::sync::ShardedLock;
use std::sync::Arc;

pub type EditorSync<'a> = Arc<ShardedLock<Editor<'a>>>;

pub struct Editor<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) target: BranchIdentifier,
    pub(super) data: Option<EditorData>,
}

pub struct EditorData {
    info: BranchInfo,
    packed_delta: Option<Delta>,
    live_changes: Vec<Patch>,
    state: State,
}

impl<'a> Editor<'a> {
    /// Init the editor, by loading the data from the DB.
    /// Init the editor, by loading the data from the DB.
    pub(super) fn open(&mut self) -> Result<()> {
        if self.data.is_some() {
            return Ok(());
        }

        let info = self
            .context
            .db
            .get(keys::Branch(&self.target))?
            .ok_or(Error::BranchNotFound)?;

        let mut state = self.context.checkout(&info.head)?;
        let packed_delta = self.context.db.get(keys::PackedDelta(&self.target))?;
        if let Some(delta) = &packed_delta {
            state.apply_delta_trusted(delta.clone());
        }

        let live_changes = self
            .context
            .db
            .get(keys::LiveChanges(&self.target))?
            .unwrap_or(Vec::new());
        for patch in live_changes.iter() {
            state
                .perform(patch.actions.clone())
                .map_err(|_| Error::CheckoutFailed)?;
        }

        self.data.replace(EditorData {
            info,
            packed_delta,
            live_changes,
            state,
        });

        Ok(())
    }

    pub fn perform(&mut self, user: &UserId, patch: Patch) {}
}
