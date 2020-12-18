use super::{context, Context};
use crate::db::keys;
use crate::error::*;
use crate::types::*;
use crossbeam::sync::ShardedLock;
use std::sync::Arc;

pub(super) type EditorSync<'a> = Arc<ShardedLock<Editor<'a>>>;

pub struct Editor<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) target: BranchOrMergeBranchId,
    pub(super) data: Option<EditorData>,
}

pub(super) struct EditorData {}

impl<'a> Editor<'a> {
    /// Init the editor, by loading the data from the DB.
    pub(super) fn open(&mut self) -> Result<()> {
        if self.data.is_some() {
            return Ok(());
        }

        match &self.target {
            BranchOrMergeBranchId::MergeBranch(id) => {}
            BranchOrMergeBranchId::Branch(id) => {}
        }

        Ok(())
    }

    pub fn perform(&mut self, user: &UserId, patch: Patch) {}
}
