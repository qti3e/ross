use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Any mutation action on a single object.
#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    /// Create a new object with the given information.
    Create { id: ObjectId, data: ObjectData },
    /// Delete an object if `version` is greater-than or equal the to the object's
    /// version.
    Delete {
        id: ObjectId,
        version: ObjectVersion,
    },
    /// Check-and-set: Compare the current value of a field with the given value,
    /// and if they are the same update the field's value to `next`.
    /// A `CAS {id, field, current: v, next: v}` style CAS can be used in transactions
    /// to ensure the value of a field upon updating another field, which can be useful
    /// to implement `enum`s (compare the enum-tag along with the variant value.)
    CAS {
        id: ObjectId,
        field: FieldId,
        current: PrimitiveValue,
        next: PrimitiveValue,
    },
}

/// An atomic batch of actions, a.k.a transaction.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionBatch {}
