use crate::action::{Object, PrimitiveValue};
use crate::ObjectID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Conflict {
    DeleteSet {
        uuid: ObjectID,
        data: Option<Object>,
    },
    CAS {
        uuid: ObjectID,
        key: String,
        next: PrimitiveValue,
        current: PrimitiveValue,
        actual: PrimitiveValue,
    },
    Collision {
        uuid: ObjectID,
    },
}
