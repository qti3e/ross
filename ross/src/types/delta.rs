use super::{set_field, FieldIndex, Object, ObjectId, ObjectVersion, PrimitiveValue};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    marker::PhantomData,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum DeltaEntry {
    Deleted,
    Inserted {
        data: Vec<PrimitiveValue>,
        version: Option<ObjectVersion>,
    },
    Updated {
        version: i16,
        changes: BTreeMap<FieldIndex, PrimitiveValue>,
    },
}

pub type Delta = HashMap<ObjectId, DeltaEntry>;

pub struct DeltaBuilder<T>(Delta, PhantomData<T>);

pub mod direction {
    pub trait Direction {
        const UNIT: i16;
    }

    pub struct Forward;
    pub struct Backward;

    impl Direction for Forward {
        const UNIT: i16 = 1;
    }

    impl Direction for Backward {
        const UNIT: i16 = -1;
    }
}

impl<T> DeltaBuilder<T>
where
    T: direction::Direction,
{
    #[inline]
    pub fn new() -> Self {
        DeltaBuilder(Delta::new(), PhantomData)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        DeltaBuilder(Delta::with_capacity(capacity), PhantomData)
    }

    #[inline]
    pub fn build(self) -> Delta {
        self.0
    }

    #[inline]
    pub fn touch(&mut self, oid: ObjectId) {
        self.0
            .entry(oid)
            .and_modify(|e| match e {
                DeltaEntry::Deleted => unreachable!(),
                DeltaEntry::Inserted { version, .. } => {
                    if T::UNIT > 0 {
                        *version.get_or_insert(0) += T::UNIT as u16;
                    } else {
                        *version.get_or_insert(0) -= (-T::UNIT) as u16;
                    }
                }
                DeltaEntry::Updated { version, .. } => {
                    *version += T::UNIT;
                }
            })
            .or_insert(DeltaEntry::Updated {
                version: T::UNIT,
                changes: BTreeMap::new(),
            });
    }

    #[inline]
    pub fn delete(&mut self, oid: ObjectId) {
        match self.0.entry(oid) {
            Entry::Occupied(mut entry) => match entry.get() {
                DeltaEntry::Deleted => {}
                DeltaEntry::Inserted { .. } => {
                    entry.remove();
                }
                DeltaEntry::Updated { .. } => {
                    entry.insert(DeltaEntry::Deleted);
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Deleted);
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, oid: ObjectId, obj: Object) {
        match self.0.entry(oid) {
            Entry::Occupied(entry) => match entry.get() {
                DeltaEntry::Deleted => {
                    entry.remove();
                }
                DeltaEntry::Inserted { .. } => unreachable!(),
                DeltaEntry::Updated { .. } => {
                    unreachable!()
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Inserted {
                    data: obj.data,
                    version: Some(obj.version),
                });
            }
        }
    }

    #[inline]
    pub fn set(&mut self, oid: ObjectId, field: FieldIndex, value: PrimitiveValue) {
        match self.0.entry(oid) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                DeltaEntry::Deleted => unreachable!(),
                DeltaEntry::Inserted { version, data } => {
                    if T::UNIT > 0 {
                        *version.get_or_insert(0) += T::UNIT as u16;
                    } else {
                        *version.get_or_insert(0) -= (-T::UNIT) as u16;
                    }
                    set_field(data, field, value);
                }
                DeltaEntry::Updated { version, changes } => {
                    *version += T::UNIT;
                    changes.insert(field, value);
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Updated {
                    version: T::UNIT,
                    changes: {
                        let mut map = BTreeMap::new();
                        map.insert(field, value);
                        map
                    },
                });
            }
        }
    }
}
