#[macro_use]
mod macros;

pub mod action;
pub mod bincode_vec_append;
pub mod branch;
pub mod commit;
pub mod conflict;
pub mod context;
pub mod db;
pub mod drop_map;
pub mod error;
pub mod hash;
pub mod log;
pub mod session;
pub mod snapshot;

pub type Timestamp = u128;
pub type UserID = hash::Hash16;
pub type RepositoryID = hash::Hash16;
pub type BranchID = hash::Hash16;
pub type CommitID = hash::Hash20;
pub type ObjectID = hash::Hash16;

pub fn now() -> Timestamp {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis()
}
