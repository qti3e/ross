use super::{EditorBox, EditorSessionId};
use crate::error::*;
use crate::types::UserId;

pub struct Session<'a, R> {
    editor: EditorBox<'a, R>,
    user: Option<UserId>,
    id: EditorSessionId,
}

impl<'a, R> Session<'a, R> {
    pub fn new(editor: EditorBox<'a, R>, user: Option<UserId>, sender: R) -> Result<Self> {
        let id = editor
            .write()
            .map_err(|_| Error::AcquireWriteLock)?
            .connect(sender);
        Ok(Session { editor, user, id })
    }

    pub fn perform() {}
}

impl<'a, R> Drop for Session<'a, R> {
    fn drop(&mut self) {
        if let Ok(mut editor) = self.editor.write().map_err(|_| Error::AcquireWriteLock) {
            editor.disconnect(&self.id);
        }
    }
}
