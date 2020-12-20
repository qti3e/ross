use super::{FieldIndex, ObjectId, ObjectVersion, PrimitiveValue, Timestamp, UserId};
use serde::{Deserialize, Serialize};

pub type ActionId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchAtom {
    /// Touch an object to ensure that it still exists and also increments its
    /// version in order to prevent further delayed delete action to take place.  
    /// It's generally used when we reference the object in another object.
    /// # Example
    /// ```txt
    /// --- Types
    /// struct Scene { title: string; }
    /// struct Box in Scene as .boxes { size: num }
    /// --- Patch
    /// Insert(Box) { owner: S1, size: 100 }
    /// Touch(S1) # Because we don't want S1 to be deleted when this action is happening.
    /// ```
    Touch { oid: ObjectId },
    /// Create a new object in the database with the given information.
    Insert {
        /// A unique ID for this object, an `PatchConflict::IdConflict` is returned if
        /// an object with the same id already exists.
        oid: ObjectId,
        /// Vector containing the information about the object, it should not contain
        /// more than 256 items. (it's indexed using u8.)
        data: Vec<PrimitiveValue>,
        /// By default `0` is used as the version, but in case we want to restore a
        /// deleted object, we can provide the version it was deleted at.
        version: Option<ObjectVersion>,
    },
    /// A delete operation, removes an object from the state if and only if its version
    /// is not greater than the provided version.
    Delete {
        /// Id of the object to be removed.
        oid: ObjectId,
        /// Maximum version of the object we're allowed to delete, if version of the
        /// object currently in the stater is newer than this provided version a
        /// `PatchConflict::WriteDelete` is returned.
        version: ObjectVersion,
    },
    /// Compare-And-Set operation, set the value of a field inside an object if and
    /// only if the current value is intact.
    CAS {
        /// Id of the object.
        oid: ObjectId,
        /// Index in data-vector.
        field: FieldIndex,
        /// The `expected` value.
        current: PrimitiveValue,
        /// The `next` value, if the `actual` value is equal to `target` the object
        /// remains untouched.
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
