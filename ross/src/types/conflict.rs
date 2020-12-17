use super::{FieldIndex, Object, ObjectId, PrimitiveValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MergeConflict {
    WriteWrite {
        oid: ObjectId,
        field: FieldIndex,
        origin: PrimitiveValue,
        target: PrimitiveValue,
    },
    DeleteWrite {
        oid: ObjectId,
        field: FieldIndex,
        origin: PrimitiveValue,
        target: PrimitiveValue,
    },
    WriteDelete {
        oid: ObjectId,
        origin: Object,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PatchConflict {
    IdConflict { oid: ObjectId },
    WriteWrite { oid: ObjectId, field: FieldIndex },
    DeleteWrite { oid: ObjectId },
    WriteDelete { oid: ObjectId },
}
