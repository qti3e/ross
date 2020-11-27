use crate::prelude::*;
use std::collections::{HashMap, HashSet};

pub struct CompactDelta {
    pub deleted: HashSet<ObjectId>,
    pub insert: HashMap<ObjectId, ObjectData>,
    pub field_change: HashMap<(ObjectId, FieldId), PrimitiveValue>,
}
