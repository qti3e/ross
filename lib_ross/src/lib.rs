pub mod action;
pub mod branch;
pub mod commit;
pub mod conflict;
pub mod context;
pub mod db;
pub mod hash;
pub mod log;
pub mod snapshot;

pub type Timestamp = u64;
pub type UserID = hash::Hash16;
