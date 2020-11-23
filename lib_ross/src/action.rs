use crate::hash::Hash16;
use crate::{Timestamp, UserID};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

#[derive(Debug, Serialize, Deserialize)]
pub enum PrimitiveValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

/// Any single action that can mutate the data.
#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    CREATE {
        uuid: Hash16,
        instance: String,
        data: Value,
    },
    DELETE {
        uuid: Hash16,
    },
    CAS {
        uuid: Hash16,
        key: String,
        current: PrimitiveValue,
        next: PrimitiveValue,
    },
}

/// A set of atomic actions, either all of the actions happen or none happens.
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    /// A set of actions that must be performed in this transaction.
    pub actions: Vec<Action>,
    /// The time in which the transaction was authored.
    pub time: Timestamp,
    /// The user who authored the transaction.
    pub user: UserID,
    /// An optional value that can be used to name the action, users of this
    /// library can have named transactions, like `createUser`, this value can
    /// be used to assign a unique id to each named action and then be used to
    /// format the transaction.
    pub action: Option<u16>,
}
