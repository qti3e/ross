use std::time::{SystemTime, UNIX_EPOCH};

pub type Timestamp = u128;

/// Return the current unix timestamp.
pub fn now() -> Timestamp {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis()
}
