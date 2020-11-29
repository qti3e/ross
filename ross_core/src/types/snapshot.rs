use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize)]
pub enum SnapshotRef {
    Snapshot(Snapshot),
    DiffSnapshot(CommitIdentifier, CompactDelta),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot(HashMap<ObjectId, Object>);

impl Snapshot {
    /// Compute the diff of this snapshot relative to a base, note that in order for
    /// this method to work properly, `base` must be an ancestor of the current snapshot.
    pub fn diff_relative_to_base(&self, base: &Snapshot) -> CompactDelta {
        let mut delta = CompactDelta::default();
        let mut unvisited: HashSet<&ObjectId> = self.0.keys().collect();

        for (id, (base_version, base_data)) in &base.0 {
            if unvisited.remove(id) {
                let (version, data) = self.0.get(id).unwrap();
                // This snapshot is derived from the base, so if the versions are the same
                // that means data is untouched.
                if version == base_version {
                    continue;
                }
                // Figure out what has changed.
                for (field, (base, current)) in base_data.iter().zip(data.iter()).enumerate() {
                    if base == current {
                        continue;
                    }
                    delta.set(*id, field as u8, current.clone());
                }
                continue;
            }
            delta.create(*id, base_data.clone())
        }

        for id in unvisited.into_iter() {
            delta.delete(id.clone());
        }

        delta
    }

    /// Part of the `apply_batch_patch`, but in it's separated to ensure that it's not mutable,
    /// and can not change the state in case there are conflicts. (it's not `&mut self`.)
    #[inline]
    fn collect_conflicts(&self, batch: &BatchPatch) -> Vec<PatchConflict> {
        let mut conflicts = Vec::<PatchConflict>::new();

        for patch in &batch.patches {
            match patch {
                Patch::Create { id, .. } => {
                    if self.0.contains_key(id) {
                        conflicts.push(PatchConflict::IdCollision { id: *id });
                    }
                }
                Patch::Delete { id, version } => {
                    if let Some((ver, _)) = self.0.get(id) {
                        if ver > version {
                            conflicts.push(PatchConflict::WriteDelete { id: *id });
                        }
                    }
                }
                Patch::CAS {
                    id,
                    field,
                    base,
                    target,
                } => {
                    if let Some((_, data)) = self.0.get(id) {
                        let actual = data.get(*field);

                        if &actual == base || &actual == target {
                            continue;
                        }

                        conflicts.push(PatchConflict::CAS {
                            id: *id,
                            field: *field,
                        });
                    } else {
                        conflicts.push(PatchConflict::DeleteWrite { id: *id });
                    }
                }
            }
        }

        conflicts
    }

    /// Apply an atomic batch of updates or return the conflicts that prevented the transaction
    //// to finish.
    pub fn apply_batch_patch(&mut self, batch: &BatchPatch) -> Option<Vec<PatchConflict>> {
        let conflicts = self.collect_conflicts(batch);
        if !conflicts.is_empty() {
            return Some(conflicts);
        }

        for patch in &batch.patches {
            match patch {
                Patch::Create { id, data } => {
                    self.0.insert(*id, (0, data.clone()));
                }
                Patch::Delete { id, .. } => {
                    self.0.remove(id);
                }
                Patch::CAS {
                    id, field, target, ..
                } => {
                    let obj = self.0.get_mut(id).unwrap();
                    obj.1.set(*field, target.clone());
                }
            }
        }

        None
    }
}
