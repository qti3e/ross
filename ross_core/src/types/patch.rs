use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A patch is a diff that contains information about the base.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Patch {
    Create {
        id: ObjectId,
        data: ObjectData,
    },
    Delete {
        id: ObjectId,
        version: ObjectVersion,
    },
    CAS {
        id: ObjectId,
        field: FieldId,
        base: PrimitiveValue,
        target: PrimitiveValue,
    },
}

/// An atomic batch of patches.
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchPatch {
    pub patches: Vec<Patch>,
    pub author: UserId,
    pub action: ActionKind,
    pub time: Timestamp,
}
