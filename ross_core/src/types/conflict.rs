use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A raw conflict.
#[derive(Debug, Serialize, Deserialize)]
pub enum PatchConflict {
    IdCollision { id: ObjectId },
    WriteDelete { id: ObjectId },
    DeleteWrite { id: ObjectId },
    CAS { id: ObjectId, field: FieldId },
}
