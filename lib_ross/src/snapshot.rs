use crate::action::{Action, Object, PrimitiveValue};
use crate::conflict::Conflict;
use crate::hash::Hash16;
use rpds::RedBlackTreeMapSync;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

/// Snapshot is a immutable object that contains all of the key-value pairs
/// at a certain time, a branch or a commit.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot(RedBlackTreeMapSync<Hash16, SnapshotObject>);

// Like `action::Object` but immutable.
pub type SnapshotObject = RedBlackTreeMapSync<String, PrimitiveValue>;

impl Snapshot {
    pub fn get_object(&self, uuid: &Hash16) -> Option<&SnapshotObject> {
        self.0.get(uuid)
    }

    /// Perform a batch of actions on the snapshot, returns a new snapshot with
    /// the changes applied or a list of conflicts that prevented the transaction
    /// to finish.
    pub fn perform(&self, actions: &Vec<Action>) -> Result<Self, Vec<Conflict>> {
        let mut conflicts: Vec<Conflict> = Vec::new();
        let mut has_conflict = false;

        let mut map = self.0.clone();
        for action in actions {
            let (obj, uuid, key, current, next) = match action {
                Action::CREATE { uuid, .. } if map.contains_key(uuid) => {
                    conflicts.push(Conflict::Collision { uuid: uuid.clone() });
                    continue;
                }
                Action::CREATE { uuid, data } if !has_conflict => {
                    let iter = data.iter().map(|(k, v)| (k.clone(), v.clone()));
                    let data_imu = RedBlackTreeMapSync::from_iter(iter);
                    map.insert_mut(uuid.clone(), data_imu);
                    continue;
                }
                Action::DELETE { uuid } if !has_conflict => {
                    map.remove_mut(uuid);
                    continue;
                }
                Action::CAS {
                    uuid,
                    key,
                    current,
                    next,
                } => match map.get(uuid) {
                    Some(obj) => (
                        obj,
                        uuid.clone(),
                        key.clone(),
                        current.clone(),
                        next.clone(),
                    ),
                    None => match self.get_object(uuid) {
                        Some(org) => {
                            let data: Object =
                                org.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                            has_conflict = true;
                            conflicts.push(Conflict::DeleteSet {
                                uuid: uuid.clone(),
                                data: Some(data),
                            });
                            continue;
                        }
                        None => {
                            has_conflict = true;
                            conflicts.push(Conflict::DeleteSet {
                                uuid: uuid.clone(),
                                data: None,
                            });
                            continue;
                        }
                    },
                },
                _ => continue,
            };

            let actual = match obj.get(&key) {
                Some(data) => data.clone(),
                None => PrimitiveValue::Null,
            };

            if actual != current {
                has_conflict = true;
                conflicts.push(Conflict::CAS {
                    uuid,
                    key,
                    next,
                    current,
                    actual,
                });
            } else if !has_conflict {
                let new_obj = obj.insert(key, next);
                map.insert_mut(uuid, new_obj);
            }
        }

        match has_conflict {
            true => Err(conflicts),
            false => Ok(Snapshot(map)),
        }
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Snapshot(RedBlackTreeMapSync::new_sync())
    }
}
