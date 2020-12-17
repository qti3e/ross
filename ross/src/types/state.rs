use super::{
    direction, Delta, DeltaBuilder, DeltaEntry, Patch, PatchAtom, PatchConflict, PrimitiveValue,
};
use crate::utils::hash::Hash16;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};

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
                    self.objects.insert(
                        id,
                        Object {
                            version: version.unwrap_or(0),
                            data,
                        },
                    );
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
    pub fn perform(&mut self, patch: &Patch) -> Result<Delta, Vec<PatchConflict>> {
        let mut revert_builder =
            DeltaBuilder::<direction::Backward>::with_capacity(patch.actions.len() / 2);
        let mut perform = true; // basically `conflicts.len() == 0`.
        let mut conflicts = Vec::new();

        for atom in &patch.actions {
            match atom {
                PatchAtom::Touch { oid } => {
                    match self.objects.get_mut(oid) {
                        Some(obj) if perform => {
                            revert_builder.touch(*oid);
                            obj.version += 1;
                        }
                        Some(_) => {}
                        None => {
                            // Delete-Write
                            perform = false;
                            conflicts.push(PatchConflict::DeleteWrite { oid: *oid });
                        }
                    }
                }
                PatchAtom::Insert { oid, data, version } => {
                    match self.objects.entry(*oid) {
                        Entry::Occupied(_) => {
                            // ID-conflict.
                            perform = false;
                            conflicts.push(PatchConflict::IdConflict { oid: *oid });
                        }
                        Entry::Vacant(entry) => {
                            revert_builder.delete(*oid);
                            entry.insert(Object {
                                version: version.unwrap_or(0),
                                data: data.clone(),
                            });
                        }
                    }
                }
                PatchAtom::Delete { oid, version } => {
                    match self.objects.entry(*oid) {
                        Entry::Occupied(entry) if &entry.get().version > version => {
                            // Write-Delete conflict.
                            perform = false;
                            conflicts.push(PatchConflict::WriteDelete { oid: *oid });
                        }
                        Entry::Occupied(entry) if perform => {
                            let obj = entry.remove();
                            revert_builder.insert(*oid, obj);
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
                    match self.objects.get_mut(oid) {
                        Some(obj) => {
                            match get_field(&obj.data, *field) {
                                v if v == target => { /* Already there */ }
                                v if v == current => {
                                    if perform {
                                        obj.version += 1;
                                        let prev = set_field(&mut obj.data, *field, target.clone());
                                        revert_builder.set(*oid, *field, prev);
                                    }
                                }
                                _ => {
                                    // Write-Write
                                    perform = false;
                                    conflicts.push(PatchConflict::WriteWrite {
                                        oid: *oid,
                                        field: *field,
                                    });
                                }
                            }
                        }
                        None => {
                            // Delete-Write
                            perform = false;
                            conflicts.push(PatchConflict::DeleteWrite { oid: *oid });
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
pub(super) fn set_field(
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
