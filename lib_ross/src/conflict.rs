use crate::action::{Object, PrimitiveValue};
use crate::hash::Hash16;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Conflict {
    DeleteSet {
        uuid: Hash16,
        data: Option<Object>,
    },
    CAS {
        uuid: Hash16,
        key: String,
        next: PrimitiveValue,
        current: PrimitiveValue,
        actual: PrimitiveValue,
    },
    Collision {
        uuid: Hash16,
    },
}
