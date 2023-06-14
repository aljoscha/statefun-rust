use crate::{EgressRecord, MyUserProfile, DelayedMessage, UserLogin};
use statefun::{TypeSpec, ValueSpec};

pub fn seen_count_spec() -> ValueSpec<i32> {
    ValueSpec::<i32>::new("seen_count")
}

pub fn is_first_visit_spec() -> ValueSpec<bool> {
    ValueSpec::<bool>::new("is_first_visit")
}

pub fn last_seen_timestamp_spec() -> ValueSpec<i64> {
    ValueSpec::<i64>::new("last_seen_timestamp")
}

pub fn delayed_message_type_spec() -> TypeSpec<DelayedMessage> {
    TypeSpec::<DelayedMessage>::new()
}

pub fn user_profile_type_spec() -> TypeSpec<MyUserProfile> {
    TypeSpec::<MyUserProfile>::new()
}

pub fn user_login_type_spec() -> TypeSpec<UserLogin> {
    TypeSpec::<UserLogin>::new()
}

// note: the playground image actually hardcodes this check so we have to match it for now,
// until we configure our own playground
pub fn egress_record_type_spec() -> TypeSpec<EgressRecord> {
    TypeSpec::<EgressRecord>::new()
}
