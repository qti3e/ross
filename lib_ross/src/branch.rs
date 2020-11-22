use crate::hash::{Hash16, Hash20};
use crate::{Timestamp, UserID};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BranchIdentifier {
  pub project: Hash16,
  pub uuid: Hash16
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
  head: Hash20,
  branch_root: Option<Hash20>,
  date: Timestamp,
  user: UserID,
  is_static: bool,
  is_archived: bool,
  name: String
}
