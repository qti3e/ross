use super::EditorSync;
use crate::types::UserId;

pub struct Session<'a> {
    pub(super) editor: EditorSync<'a>,
    /// A readonly session.
    pub(super) user: Option<UserId>,
}
