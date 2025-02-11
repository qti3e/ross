use super::{Editor, EditorBox, EditorLock, Recipient, Session};
use crate::db::{keys, DB};
use crate::error::*;
use crate::types::*;
use crate::utils::clock::now;
use crate::utils::ttl_map::TTLMap;
use std::sync::Mutex;

pub struct Context<'a, R> {
    pub(super) db: DB,
    editors: Mutex<TTLMap<BranchIdentifier, EditorLock<'a, R>>>,
}

impl<'a, R> Context<'a, R> {
    pub fn new(path: &str) -> Self {
        Self {
            db: DB::open(path),
            editors: Mutex::new(TTLMap::new(10, 60000)),
        }
    }

    /// Returns the snapshot of a commit.
    #[inline]
    pub fn checkout(&self, commit: &CommitIdentifier) -> Result<State> {
        match self
            .db
            .get(keys::CommitSnapshot(commit))?
            .ok_or(Error::CommitNotFound)?
        {
            SnapshotEntry::Snapshot(state) => Ok(state),
            SnapshotEntry::Delta { base, delta } => {
                let mut state = self.checkout(&base)?;
                state.apply_delta_trusted(delta);
                Ok(state)
            }
        }
    }

    /// Open a new a session on the given branch/merge-branch, a session can be used
    /// to edit/see a branch.
    pub fn open_session(
        &'a self,
        target: BranchIdentifier,
        user: Option<UserId>,
        sender: R,
    ) -> Result<Session<'a, R>>
    where
        R: Recipient,
    {
        let editor = {
            // This code is placed in {} intentionally, we want to release the mutex
            // as soon as possible.
            let mut editors = self.editors.lock().map_err(|_| Error::AcquireLock)?;
            let editor = editors.get_or_maybe_insert_with(target, || {
                Ok(EditorLock::new(Editor::new(&self, target)))
            })?;
            editor.clone()
        };

        // If it' the first time we're accessing this editor, call the open.
        // 2 = in ttl_map + current reference (`editor`).
        if editor.strong_count() == 2 {
            editor
                .write()
                .map_err(|_| Error::AcquireWriteLock)?
                .open()?;
        }

        Ok(Session::new(EditorBox::new(editor), user, sender)?)
    }

    #[inline]
    pub(super) fn drop_editor(&self, target: BranchIdentifier) {
        if let Ok(mut editors) = self.editors.lock() {
            editors.drop_item(target, now());
        }
    }
}

fn t() {
    use crate::utils::hash::Hash16;
    let ctx = Context::<()>::new("path-xxx");
    let s1 = ctx
        .open_session(
            BranchIdentifier {
                repository: RepositoryId(Hash16::MAX),
                id: BranchId(Hash16::MIN),
            }
            .into(),
            None,
            (),
        )
        .unwrap();
    let s2 = ctx
        .open_session(
            BranchIdentifier {
                repository: RepositoryId(Hash16::MAX),
                id: BranchId(Hash16::MIN),
            }
            .into(),
            None,
            (),
        )
        .unwrap();
}
