use statefun::{ValueSpec, Expiration, ExpirationType};
use std::time::Duration;

// 'seen_count' will automatically be purged 5 seconds after the last write
pub fn seen_count_spec() -> ValueSpec<i32> {
    ValueSpec::<i32>::new("seen_count", Expiration::new(ExpirationType::AfterWrite, Duration::from_secs(5)))
}

pub fn is_first_visit_spec() -> ValueSpec<bool> {
    ValueSpec::<bool>::new("is_first_visit", Expiration::never())
}

pub fn last_seen_timestamp_spec() -> ValueSpec<i64> {
    ValueSpec::<i64>::new("last_seen_timestamp", Expiration::never())
}
