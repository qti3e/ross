use crate::prelude::*;

/// The random id of the object.
pub type ObjectId = Hash16;

/// Version of the object, each transaction increments the version by 1, the version
/// might not seem necessary but it is used to detect write-delete conflicts.
pub type ObjectVersion = u32;

/// The object that is stored in the database, usually an object is shaped
/// by a key -> value map, and we also use String keys in `ross` but, here
/// in `ross_core` we flatten each object into a vector, the `ross_compiler`
/// should take care of converting each key of types into a number, encoding
/// and decoding the tuple returned by `core` into an actual object.
pub type Object = (ObjectVersion, ObjectData);

/// The data of an object as explained in `Object`.
pub type ObjectData = Vec<PrimitiveValue>;

/// The field of the object, it is the index of an item in Object.1#Vec.
/// u8 might seem small because usually a usize is used for indexing
/// vectors, but I believe 256 fields is a lot of fields.
pub type FieldId = u8;
