use super::{FieldIndex, ObjectId, ObjectVersion, PrimitiveValue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize)]
pub enum DeltaEntry {
    Deleted,
    Inserted {
        data: Vec<PrimitiveValue>,
        version: ObjectVersion,
    },
    Updated {
        version: i16,
        changes: BTreeMap<FieldIndex, PrimitiveValue>,
    },
}

pub type Delta = HashMap<ObjectId, DeltaEntry>;
