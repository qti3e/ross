use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Conflict {
    DeleteSet {
        uuid: ObjectID,
        data: Option<action::Object>,
    },
    CAS {
        uuid: ObjectID,
        key: String,
        next: action::PrimitiveValue,
        current: action::PrimitiveValue,
        actual: action::PrimitiveValue,
    },
    Collision {
        uuid: ObjectID,
    },
}

/// The `JsonConflict` contains the same information as the `Conflict` with one difference,
/// hashes (such as ObjectID) are stored as String, this struct should be used to report
/// the conflicts to the end user.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonConflict {
    DeleteSet {
        uuid: String,
        data: Option<action::Object>,
    },
    CAS {
        uuid: String,
        key: String,
        next: action::PrimitiveValue,
        current: action::PrimitiveValue,
        actual: action::PrimitiveValue,
    },
    Collision {
        uuid: String,
    },
}

impl From<&Conflict> for JsonConflict {
    fn from(data: &Conflict) -> Self {
        match data {
            Conflict::DeleteSet { uuid, data } => JsonConflict::DeleteSet {
                uuid: String::from(uuid),
                data: data.clone(),
            },
            Conflict::CAS {
                uuid,
                key,
                next,
                current,
                actual,
            } => JsonConflict::CAS {
                uuid: String::from(uuid),
                key: key.clone(),
                next: next.clone(),
                current: current.clone(),
                actual: actual.clone(),
            },
            Conflict::Collision { uuid } => JsonConflict::Collision {
                uuid: String::from(uuid),
            },
        }
    }
}
