use super::Editor;
use crossbeam::sync::{ShardedLock, ShardedLockReadGuard, ShardedLockWriteGuard};
use std::ops::Deref;
use std::sync::{Arc, LockResult, TryLockResult};

/// A reference counted RW-lock over an Editor instance, instances of this
/// type are owned by the context, and they're stored in the ttl_map.  
/// You should use [EditorBox](EditorBox) when exposing the editor via the
/// public API.
#[derive(Clone)]
pub struct EditorLock<'a, R>(Arc<ShardedLock<Editor<'a, R>>>);

impl<'a, R> EditorLock<'a, R> {
    #[inline]
    pub(super) fn new(editor: Editor<'a, R>) -> Self {
        EditorLock(Arc::new(ShardedLock::new(editor)))
    }

    /// Gets the number of strong (`Arc`) pointers to this editor.
    #[inline(always)]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }

    /// Attempts to acquire this lock with exclusive write access.
    #[inline(always)]
    pub fn write(&self) -> LockResult<ShardedLockWriteGuard<Editor<'a, R>>> {
        self.0.write()
    }

    /// Locks with shared read access, blocking the current thread until it can be acquired.
    #[inline(always)]
    pub fn read(&self) -> LockResult<ShardedLockReadGuard<Editor<'a, R>>> {
        self.0.read()
    }

    /// Attempts to acquire this lock with exclusive write access.
    /// If the access could not be granted at this time, an error is returned.
    #[inline(always)]
    pub fn try_write(&self) -> TryLockResult<ShardedLockWriteGuard<Editor<'a, R>>> {
        self.0.try_write()
    }

    /// Attempts to acquire this lock with shared read access.
    /// If the access could not be granted at this time, an error is returned.
    #[inline(always)]
    pub fn try_read(&self) -> TryLockResult<ShardedLockReadGuard<Editor<'a, R>>> {
        self.0.try_read()
    }
}

/// An `EditorLock` that does not live inside the context.
pub struct EditorBox<'a, R>(EditorLock<'a, R>);

/// A wrapper around [EditorLock](EditorLock), this type should be used when
/// exposing an editor instance via the public API.  
/// It watches the reference counts and closes the editor (by removing it from
/// the context) when there are no more references to the data.  
/// This type implements `Deref<Target = EditorLock>` so it can be treated as
/// an `EditorLock`.
impl<'a, R> EditorBox<'a, R> {
    #[inline]
    pub fn new(lock: EditorLock<'a, R>) -> Self {
        EditorBox(lock)
    }
}

impl<'a, R> Deref for EditorBox<'a, R> {
    type Target = EditorLock<'a, R>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, R> Drop for EditorBox<'a, R> {
    fn drop(&mut self) {
        if self.0.strong_count() == 2 {
            if let Ok(editor) = self.0.read() {
                editor.context.drop_editor(editor.target);
            }
        }
    }
}
