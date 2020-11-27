use crate::db_keys;
use serde::{Deserialize, Serialize};

db_keys!(DBKey(Key) {
  Name(u8) -> String,
  Aliases(u8) -> Vec<String>
});

db_partial_keys!(DBKey(Key)::PartialDBKey {
  Name(u8)::NameLength -> usize,
  Aliases(u8)::NumberOfAliases -> usize
});
