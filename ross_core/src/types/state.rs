use super::{Delta, DeltaEntry, PatchAtom, PatchConflict, PrimitiveValue};
use crate::utils::hash::Hash16;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, BTreeMap, HashMap};

/// ID of an object.
pub type ObjectId = Hash16;

/// Each object in ROSS is stored as a vector of PrimitiveValues.
pub type FieldIndex = u8;

/// In ross objects are versioned.
pub type ObjectVersion = u16;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub version: ObjectVersion,
    pub data: Vec<PrimitiveValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    objects: HashMap<ObjectId, Object>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
}

impl State {
    /// Apply a trusted diff to turn this state into the next.
    ///
    /// # Panics
    /// If there is an `Updated` delta for an object that does not exists.
    pub fn apply_delta_trusted(&mut self, delta: Delta) {
        for (id, entry) in delta {
            match entry {
                DeltaEntry::Deleted => {
                    self.objects.remove(&id);
                }
                DeltaEntry::Inserted { data, version } => {
                    self.objects.insert(id, Object { version, data });
                }
                DeltaEntry::Updated { version, changes } => {
                    let obj = self.objects.get_mut(&id).unwrap();
                    obj.version = if version > 0 {
                        obj.version + version as u16
                    } else {
                        obj.version - (-version) as u16
                    };
                    for (field, value) in changes {
                        set_field(&mut obj.data, field, value);
                    }
                }
            }
        }
    }

    /// Performs a `Patch`, this is an atomic method, after the call either all of the
    /// purposed changes are applied or none of them. On success this method will return
    /// a trusted `Delta` which can later be used to revert the changes.  
    /// The execution order of reverts is important, you are only allowed to call
    /// `apply_delta_trusted` at the same state that this method was returned.  
    /// ```txt
    /// let r1 = perform(patch1).unwrap();
    /// let r2 = perform(patch2).unwrap();
    /// // Now we have to call `apply_delta_trusted` in reverse order:
    /// apply_delta_trusted(r2);
    /// apply_delta_trusted(r1);
    /// ```
    /// On failure this method will return the list of `Conflict`s that prevented this
    /// patch to be applied.
    pub fn perform<P: IntoIterator<Item = PatchAtom>>(
        &mut self,
        patch: P,
    ) -> Result<Delta, Vec<PatchConflict>> {
        let iter = patch.into_iter();
        let mut revert_builder = RevertDeltaBuilder::with_capacity(iter.size_hint().0 / 2);
        let mut perform = true; // basically `conflicts.len() == 0`.
        let mut conflicts = Vec::new();

        for atom in iter {
            match atom {
                PatchAtom::Touch { oid } => {
                    match self.objects.get_mut(&oid) {
                        Some(obj) if perform => {
                            if revert_builder.touch(oid) {
                                obj.version += 1;
                            }
                        }
                        Some(_) => {}
                        None => {
                            // Delete-Write
                            perform = false;
                            conflicts.push(PatchConflict::DeleteWrite { oid });
                        }
                    }
                }
                PatchAtom::Insert { oid, data, version } => {
                    match self.objects.entry(oid) {
                        Entry::Occupied(_) => {
                            // ID-conflict.
                            perform = false;
                            conflicts.push(PatchConflict::IdConflict { oid });
                        }
                        Entry::Vacant(entry) => {
                            if let Some(obj) = revert_builder.delete(oid) {
                                // don't trust the user in this case.
                                entry.insert(obj);
                            } else {
                                entry.insert(Object {
                                    version: version.unwrap_or(0),
                                    data: data.clone(),
                                });
                            }
                        }
                    }
                }
                PatchAtom::Delete { oid, version } => {
                    match self.objects.entry(oid) {
                        Entry::Occupied(entry) if entry.get().version > version => {
                            // Write-Delete conflict.
                            perform = false;
                            conflicts.push(PatchConflict::WriteDelete { oid });
                        }
                        Entry::Occupied(entry) if perform => {
                            let obj = entry.remove();
                            revert_builder.insert(oid, obj);
                        }
                        _ => {}
                    }
                }
                PatchAtom::CAS {
                    oid,
                    field,
                    current,
                    target,
                } => {
                    match self.objects.get_mut(&oid) {
                        Some(obj) => {
                            match get_field(&obj.data, field) {
                                v if v == &target => { /* Already there */ }
                                v if v == &current => {
                                    if perform {
                                        let prev = set_field(&mut obj.data, field, target.clone());
                                        if revert_builder.set(oid, field, prev) {
                                            obj.version += 1;
                                        }
                                    }
                                }
                                _ => {
                                    // Write-Write
                                    perform = false;
                                    conflicts.push(PatchConflict::WriteWrite { oid, field });
                                }
                            }
                        }
                        None => {
                            // Delete-Write
                            perform = false;
                            conflicts.push(PatchConflict::DeleteWrite { oid });
                        }
                    }
                }
            }
        }

        if !perform {
            self.apply_delta_trusted(revert_builder.build());
            Err(conflicts)
        } else {
            Ok(revert_builder.build())
        }
    }
}

#[inline]
fn set_field(
    data: &mut Vec<PrimitiveValue>,
    field: u8,
    mut value: PrimitiveValue,
) -> PrimitiveValue {
    let field = field as usize;
    if field < data.len() {
        let current = data.get_mut(field).unwrap();
        std::mem::swap(current, &mut value);
        value
    } else {
        while field > data.len() {
            data.push(PrimitiveValue::Null);
        }
        data.push(value);
        PrimitiveValue::Null
    }
}

#[inline]
fn get_field(data: &Vec<PrimitiveValue>, field: u8) -> &PrimitiveValue {
    let field = field as usize;
    if field < data.len() {
        return &data[field];
    }
    &PrimitiveValue::Null
}

/// Used in `perform()` to build the delta used to inverse the patch.
struct RevertDeltaBuilder(Delta);

impl RevertDeltaBuilder {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        RevertDeltaBuilder(Delta::with_capacity(capacity))
    }

    #[inline]
    pub fn build(self) -> Delta {
        self.0
    }

    /// Returns `false` if the object was inserted in this patch, so that we wouldn't
    /// increase the version.
    #[inline]
    pub fn touch(&mut self, oid: ObjectId) -> bool {
        match self.0.entry(oid) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                DeltaEntry::Deleted => {
                    return false;
                }
                // The object was removed from the state in the patch, so even if it's
                // touched, a conflict is reported before this function is called, so
                // therefore this branch is unreachable.
                DeltaEntry::Inserted { .. } => unreachable!(),
                DeltaEntry::Updated { version, .. } => {
                    *version -= 1;
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Updated {
                    version: -1,
                    changes: BTreeMap::new(),
                });
            }
        }
        true
    }

    /// Set the object's status to `deleted`, if the object was deleted in this same
    /// patch the result is returned.  
    /// It is used to protect the server against `Delete-Insert` attack. Imagine the
    /// following patch is sent by a user, the user can insert any arbitrary data data
    /// after the `Delete` or even reset the version (which will break the synchronization
    /// service.), In this special case we don't trust the user with the data (`Data`) and
    /// will rely on the data returned by this function.
    /// ```txt
    ///   Delete(ID)
    ///   Insert(ID, Data)
    /// ```
    #[inline]
    pub fn delete(&mut self, oid: ObjectId) -> Option<Object> {
        match self.0.entry(oid) {
            Entry::Occupied(mut entry) => match entry.get() {
                DeltaEntry::Deleted => unreachable!(),
                DeltaEntry::Inserted { .. } => {
                    if let DeltaEntry::Inserted { version, data } = entry.remove() {
                        return Some(Object { data, version });
                    }
                }
                DeltaEntry::Updated { .. } => {
                    entry.insert(DeltaEntry::Deleted);
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Deleted);
            }
        }
        None
    }

    #[inline]
    pub fn insert(&mut self, oid: ObjectId, obj: Object) {
        match self.0.entry(oid) {
            Entry::Occupied(mut entry) => match entry.get() {
                DeltaEntry::Deleted => {
                    // Cancel-out.
                    entry.remove();
                }
                // If the DeltaEntry is Inserted, that means the object is already
                // deleted in the patch, which basically means this function never
                // gets called.
                DeltaEntry::Inserted { .. } => unreachable!(),
                DeltaEntry::Updated { .. } => {
                    entry.insert(DeltaEntry::Inserted {
                        data: obj.data,
                        version: obj.version,
                    });
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Inserted {
                    data: obj.data,
                    version: obj.version,
                });
            }
        }
    }

    /// Returns `false` if the object was inserted in this patch, so that we wouldn't
    /// increase the version.
    #[inline]
    pub fn set(&mut self, oid: ObjectId, field: FieldIndex, value: PrimitiveValue) -> bool {
        match self.0.entry(oid) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                DeltaEntry::Deleted => {
                    return false;
                }
                DeltaEntry::Inserted { .. } => unreachable!(),
                DeltaEntry::Updated { version, changes } => {
                    *version -= 1;
                    changes.insert(field, value);
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(DeltaEntry::Updated {
                    version: -1,
                    changes: {
                        let mut map = BTreeMap::new();
                        map.insert(field, value);
                        map
                    },
                });
            }
        }
        true
    }
}
