use super::{FieldIndex, ObjectId, ObjectVersion, PrimitiveValue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize)]
pub enum DeltaEntry {
    Delete,
    Update {
        new_version: ObjectVersion,
        changes: BTreeMap<FieldIndex, PrimitiveValue>,
    },
}

pub type Delta = HashMap<ObjectId, DeltaEntry>;
