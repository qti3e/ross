use super::PrimitiveValue;
use crate::utils::hash::Hash16;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ID of an object.
pub type ObjectId = Hash16;

/// Each object in ROSS is stored as a vector of PrimitiveValues.
pub type FieldIndex = u8;

/// In ross objects are versioned.
pub type ObjectVersion = u16;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub version: ObjectVersion,
    pub data: Vec<PrimitiveValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    objects: HashMap<ObjectId, Object>,
}
