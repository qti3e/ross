use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A diff between two versions of the state.
#[derive(Debug, Serialize, Deserialize)]
pub struct CompactDelta {
    /// Id of all the objects that are deleted.
    pub deleted: Vec<ObjectId>,
    /// All the new objects that are inserted.
    pub inserted: Vec<(ObjectId, ObjectData)>,
    /// Map each (key, field) into the new value that has changed.
    pub field_change: Vec<(ObjectId, FieldId, PrimitiveValue)>,
}

impl Default for CompactDelta {
    fn default() -> Self {
        CompactDelta {
            deleted: Vec::new(),
            inserted: Vec::new(),
            field_change: Vec::new(),
        }
    }
}
