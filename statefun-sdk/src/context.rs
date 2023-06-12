use statefun_proto::request_reply::Address as ProtoAddress;
use crate::ValueSpecBase;
use crate::Address;
use crate::ValueSpec;
use std::collections::HashMap;

/// Context for a single invocation of a stateful function.
///
/// The context may be used to obtain the [Address](Address) of the function of the current
/// invocation or the calling function (if the function was invoked by another function), or to
/// access state.
#[derive(Debug)]
pub struct Context<'a> {
    pub (crate) state: &'a HashMap<ValueSpecBase, Vec<u8>>,
    self_address: &'a ProtoAddress,
    caller_address: &'a ProtoAddress,
}

impl<'a> Context<'a> {
    ///
    pub fn new(
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
    pub fn get_state<T>(&self, value_spec: ValueSpec<T>) -> Option<T> {
        let deserializer = value_spec.deserializer.clone();
        let typename = value_spec.typename.to_string();
        let state = self.state.get(&value_spec.into());
        match state {
            Some(serialized) => {
                let deserialized : T = deserializer(typename, serialized);
                Some(deserialized)
            }
            None => None
        }

        // let deserialized = (value_spec.deserializer)(state);
        // self.state_updates.push(StateUpdate::Update(
        //     value_spec.into(),
        //     serialized,
        // ));
        // None

        // todo: deserialize with user-provided serializer
        // state.and_then(|serialized_state| {
        //     let unpacked_state: Option<T> = unpack_state(value_spec, serialized_state);
        //     unpacked_state
        // })
    }
}
