use crate::prelude::*;
use std::collections::{HashMap, HashSet, BTreeMap};

/// A diff between two versions of the state.
pub struct CompactDelta {
    /// Id of all the objects that are deleted.
    pub deleted: HashSet<ObjectId>,
    /// All the new objects that are inserted.
    pub insert: HashMap<ObjectId, ObjectData>,
    /// Map each (key, field) into the new value that has changed.
    pub field_change: BTreeMap<(ObjectId, FieldId), PrimitiveValue>,
}
