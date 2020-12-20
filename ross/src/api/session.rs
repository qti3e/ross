use super::{EditorBox, Recipient, RecipientHandle};
use crate::error::*;
use crate::types::UserId;

pub struct Session<'a, R> {
    editor: EditorBox<'a, R>,
    user: Option<UserId>,
    handle: RecipientHandle,
}

impl<'a, R> Session<'a, R>
where
    R: Recipient,
{
    pub fn new(editor: EditorBox<'a, R>, user: Option<UserId>, recipient: R) -> Result<Self> {
        let handle = editor
            .write()
            .map_err(|_| Error::AcquireWriteLock)?
            .subscribe(recipient);
        Ok(Session {
            editor,
            user,
            handle,
        })
    }

    pub fn perform() {}
}

impl<'a, R> Drop for Session<'a, R> {
    fn drop(&mut self) {
        if let Ok(mut editor) = self.editor.write().map_err(|_| Error::AcquireWriteLock) {
            editor.unsubscribe(&self.handle);
        }
    }
}
