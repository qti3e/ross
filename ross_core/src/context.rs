use crate::prelude::*;
use crate::utils::drop_map::DropMap;
use crate::{sync, options};

sync!(ContextSync(Context) {});

options!(ContextOptionsBuilder(ContextOptions) {
  xy: i32 = Some(5),
  t: u8 = None
});

pub struct Context {
    pub sessions: DropMap<BranchIdentifier, SessionSync>,
}

impl Context {
    pub fn create_repository() {}
    pub fn create_branch() {}
    pub fn open_session() {}
}
