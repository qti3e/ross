use crate::action::PrimitiveValue;
use crate::hash::Hash16;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Conflict {
    DeleteSet {
        uuid: Hash16,
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
