use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Conflict {
    DeleteSet {
        uuid: ObjectID,
        data: Option<action::Object>,
    },
    CAS {
        uuid: ObjectID,
        key: String,
        next: action::PrimitiveValue,
        current: action::PrimitiveValue,
        actual: action::PrimitiveValue,
    },
    Collision {
        uuid: ObjectID,
    },
}
