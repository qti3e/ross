use super::{Context, Recipient};
use crate::db::keys;
use crate::error::*;
use crate::types::*;
use std::collections::BTreeMap;

pub struct Editor<'a, R> {
    pub(super) context: &'a Context<'a, R>,
    pub(super) target: BranchIdentifier,
    recipients: BTreeMap<RecipientId, R>,
    last_recipient_id: RecipientId,
    data: Option<EditorData>,
}

pub struct EditorData {
    info: BranchInfo,
    packed_delta: Option<Delta>,
    live_changes: Vec<Patch>,
    state: State,
}

/// An opaque type to represent a handle to a session, used in some methods
/// and returned by others.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecipientHandle(RecipientId);
type RecipientId = u16;

impl<'a, R> Editor<'a, R>
where
    R: Recipient,
{
    #[inline]
    pub(super) fn new(context: &'a Context<'a, R>, target: BranchIdentifier) -> Self {
        Editor {
            context,
            target,
            recipients: BTreeMap::new(),
            last_recipient_id: 0,
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

    /// Subscribe to the messages sent by the editor, this method will
    /// return a `RecipientHandle` which can later be used to unsubscribe
    /// from the editor.
    #[inline]
    pub fn subscribe(&mut self, recipient: R) -> RecipientHandle {
        let id = self.last_recipient_id;
        self.last_recipient_id += 1;
        self.recipients.insert(id, recipient);
        RecipientHandle(id)
    }

    pub fn perform(&mut self, user: &UserId, patch: Patch) {}
}

impl<'a, R> Editor<'a, R> {
    /// Remove a recipient from the subscriptions.
    #[inline]
    pub fn unsubscribe(&mut self, session_handle: &RecipientHandle) {
        self.recipients.remove(&session_handle.0);
    }
}
