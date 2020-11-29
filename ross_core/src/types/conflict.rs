use crate::prelude::*;

pub enum PatchConflict {
    IdCollision { id: ObjectId },
    WriteDelete { id: ObjectId },
    DeleteWrite { id: ObjectId },
    CAS { id: ObjectId, field: FieldId },
}
