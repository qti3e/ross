use super::EditorBox;
use crate::types::UserId;

pub struct Session<'a> {
    pub(super) editor: EditorBox<'a>,
    /// A readonly session.
    pub(super) user: Option<UserId>,
}
