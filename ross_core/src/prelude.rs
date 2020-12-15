pub use crate::utils::hash::{Hash16, Hash20};

// now() and `Timestamp`
pub use crate::utils::clock::*;

pub(crate) use crate::error::*;
pub use crate::types::*;

pub type UserId = Hash16;
pub type ActionKind = u32;

pub use crate::context::*;
pub use crate::editor::*;
