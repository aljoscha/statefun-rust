use crate::Address;
use crate::Expiration;
use crate::Serializable;
use crate::ValueSpec;
use crate::ValueSpecBase;
use statefun_proto::request_reply::Address as ProtoAddress;
use std::collections::HashMap;

/// Context for a single invocation of a stateful function.
///
/// The context may be used to obtain the [Address](Address) of the function of the current
/// invocation or the calling function (if the function was invoked by another function), or to
/// access state.
#[derive(Debug)]
pub struct Context<'a> {
    pub(crate) state: &'a HashMap<ValueSpecBase, Vec<u8>>,
    self_address: &'a ProtoAddress,
    caller_address: &'a ProtoAddress,
}

impl<'a> Context<'a> {
    ///
    pub(crate) fn new(
        state: &'a HashMap<ValueSpecBase, Vec<u8>>,
        self_address: &'a ProtoAddress,
        caller_address: &'a ProtoAddress,
    ) -> Self {
        Context {
            state,
            self_address,
            caller_address,
        }
    }

    /// Returns the [Address](Address) of the stateful function that is being called. This is the
    /// statefun equivalent of `self`.
    pub fn self_address(&self) -> Address {
        Address::from_proto(self.self_address)
    }

    /// Returns the [Address](Address) of the stateful function that caused this function
    /// invocation, that is, the caller.
    pub fn caller_address(&self) -> Address {
        Address::from_proto(self.caller_address)
    }

    /// Returns the state (or persisted) value that previous invocations of this stateful function
    /// might have persisted under the given name.
    /// If the state does not exist, returns None.
    /// If the state does exist but could not be deserialized, returns an error within the option.
    pub fn get_state<T: Serializable<T>>(
        &self,
        value_spec: ValueSpec<T>,
    ) -> Option<Result<T, String>> {
        let typename = value_spec.spec.typename.to_string();

        // note: Flink doesn't give us the TTL when passing existing state around,
        // so we have to leave 'expiration' to its default when doing state lookups
        let key = ValueSpecBase::new(
            value_spec.spec.name.as_str(),
            value_spec.spec.typename.as_str(),
            Expiration::never(),
        );

        let state = self.state.get(&key);
        state.map(|serialized| T::deserialize(typename, serialized))
    }
}
