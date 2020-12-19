use super::Editor;
use crossbeam::sync::{ShardedLock, ShardedLockReadGuard, ShardedLockWriteGuard};
use std::ops::Deref;
use std::sync::{Arc, LockResult};

#[derive(Clone)]
pub struct EditorLock<'a>(Arc<ShardedLock<Editor<'a>>>);

impl<'a> EditorLock<'a> {
    #[inline]
    pub fn new(editor: Editor<'a>) -> Self {
        EditorLock(Arc::new(ShardedLock::new(editor)))
    }

    #[inline]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }

    #[inline]
    pub fn write(&self) -> LockResult<ShardedLockWriteGuard<Editor<'a>>> {
        self.0.write()
    }

    #[inline]
    pub fn read(&self) -> LockResult<ShardedLockReadGuard<Editor<'a>>> {
        self.0.read()
    }
}

/// An `EditorLock` that does not live inside the context.
pub struct EditorBox<'a>(EditorLock<'a>);

impl<'a> EditorBox<'a> {
    #[inline]
    pub fn new(lock: EditorLock<'a>) -> Self {
        EditorBox(lock)
    }
}

impl<'a> Deref for EditorBox<'a> {
    type Target = EditorLock<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Drop for EditorBox<'a> {
    fn drop(&mut self) {
        if self.0.strong_count() == 2 {
            if let Ok(editor) = self.0.read() {
                editor.context.drop_editor(editor.target);
            }
        }
    }
}
