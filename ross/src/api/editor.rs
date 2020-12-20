use super::Context;
use crate::db::keys;
use crate::error::*;
use crate::types::*;
use std::collections::BTreeMap;

pub type EditorSessionId = u16;

pub struct Editor<'a, R> {
    pub(super) context: &'a Context<'a, R>,
    pub(super) target: BranchIdentifier,
    sessions: BTreeMap<EditorSessionId, R>,
    last_session_id: EditorSessionId,
    data: Option<EditorData>,
}

pub struct EditorData {
    info: BranchInfo,
    packed_delta: Option<Delta>,
    live_changes: Vec<Patch>,
    state: State,
}

impl<'a, R> Editor<'a, R> {
    #[inline]
    pub(super) fn new(context: &'a Context<'a, R>, target: BranchIdentifier) -> Self {
        Editor {
            context,
            target,
            sessions: BTreeMap::new(),
            last_session_id: 0,
            data: None,
        }
    }

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

    pub fn connect(&mut self, sender: R) -> EditorSessionId {
        let id = self.last_session_id;
        self.last_session_id += 1;
        self.sessions.insert(id, sender);
        id
    }

    pub fn disconnect(&mut self, session_id: &EditorSessionId) {
        self.sessions.remove(session_id);
    }

    pub fn perform(&mut self, user: &UserId, patch: Patch) {}
}
