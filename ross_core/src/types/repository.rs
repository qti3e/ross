use crate::prelude::*;
use serde::{Deserialize, Serialize};

pub type RepositoryId = Hash16;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub user: UserId,
    pub time: Timestamp,
    pub fork_of: Option<RepositoryId>,
}
