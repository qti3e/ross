use super::{FieldIndex, ObjectId, ObjectVersion, PrimitiveValue, Timestamp, UserId};
use serde::{Deserialize, Serialize};

pub type ActionId = u32;

#[derive(Debug, Serialize, Deserialize)]
pub enum PatchAtom {
    Touch {
        oid: ObjectId,
    },
    Insert {
        oid: ObjectId,
        data: Vec<PrimitiveValue>,
        version: Option<ObjectVersion>,
    },
    Delete {
        oid: ObjectId,
        version: ObjectVersion,
    },
    CAS {
        oid: ObjectId,
        field: FieldIndex,
        current: PrimitiveValue,
        target: PrimitiveValue,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patch {
    pub user: UserId,
    pub time: Timestamp,
    pub action: ActionId,
    pub actions: Vec<PatchAtom>,
}
