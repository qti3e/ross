use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// A diff between two versions of the state.
#[derive(Debug, Serialize, Deserialize)]
pub struct CompactDelta {
    /// Id of all the objects that are deleted.
    pub deleted: HashSet<ObjectId>,
    /// All the new objects that are inserted.
    pub insert: HashMap<ObjectId, ObjectData>,
    /// Map each (key, field) into the new value that has changed.
    pub field_change: BTreeMap<(ObjectId, FieldId), PrimitiveValue>,
}

impl CompactDelta {
    pub fn create(&mut self, id: ObjectId, data: ObjectData) {
        todo!();
    }

    pub fn delete(&mut self, id: ObjectId) {
        todo!();
    }

    pub fn set(&mut self, object: ObjectId, field: FieldId, value: PrimitiveValue) {
        todo!();
    }
}

impl Default for CompactDelta {
    fn default() -> Self {
        CompactDelta {
            deleted: HashSet::new(),
            insert: HashMap::new(),
            field_change: BTreeMap::new(),
        }
    }
}

impl From<&BatchPatch> for CompactDelta {
    fn from(batch: &BatchPatch) -> Self {
        let mut delta = CompactDelta::default();

        for patch in &batch.patches {
            match patch {
                Patch::Create { id, data } => delta.create(*id, data.clone()),
                Patch::Delete { id, .. } => delta.delete(*id),
                Patch::CAS {
                    id, field, target, ..
                } => delta.set(*id, *field, target.clone()),
            }
        }

        delta
    }
}
