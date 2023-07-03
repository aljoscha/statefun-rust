use statefun::{Expiration, ValueSpec};

// 'seen_count' will automatically be purged 5 seconds after the last write
pub fn seen_count_spec() -> ValueSpec<i32> {
    ValueSpec::<i32>::new("seen_count", Expiration::never())
}

pub fn last_seen_timestamp_spec() -> ValueSpec<i64> {
    ValueSpec::<i64>::new("last_seen_timestamp", Expiration::never())
}
