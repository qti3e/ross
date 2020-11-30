use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PatchConflict {
    IdCollision { id: ObjectId },
    WriteDelete { id: ObjectId },
    DeleteWrite { id: ObjectId },
    CAS { id: ObjectId, field: FieldId },
}
