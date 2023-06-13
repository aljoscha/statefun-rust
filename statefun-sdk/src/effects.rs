use crate::Address;
use crate::EgressIdentifier;
use crate::StateUpdate;
use crate::TypeSpec;
use crate::ValueSpec;
use crate::Serializable;
use std::time::Duration;

/// Effects (or side effects) of a stateful function invocation.
///
/// This can be used to:
///  - send messages ourselves or other stateful functions
///  - send messages to an egress
///  - update the state of this stateful function, which will be available on future invocations
#[derive(Default, Debug)]
pub struct Effects {
    pub(crate) invocations: Vec<(Address, String, Vec<u8>)>,
    pub(crate) delayed_invocations: Vec<(Address, Duration, String, Vec<u8>)>,
    pub(crate) egress_messages: Vec<(EgressIdentifier, String, Vec<u8>)>,
    pub(crate) state_updates: Vec<StateUpdate>,
}

impl Effects {
    /// Creates a new empty `Effects`.
    pub fn new() -> Effects {
        Effects {
            invocations: Vec::new(),
            delayed_invocations: Vec::new(),
            egress_messages: Vec::new(),
            state_updates: Vec::new(),
        }
    }

    /// Sends a message to the stateful function identified by the address.
    // todo: check if this needs to be valuespec in the java sdk
    pub fn send<T: Serializable>(&mut self, address: Address, type_name: TypeSpec<T>, value: &T) {
        let serialized = value.serialize(type_name.typename.to_string());
        self.invocations
            .push((address, type_name.typename.to_string(), serialized));
    }

    /// Sends a message to the stateful function identified by the address after a delay.
    pub fn send_after<T: Serializable>(
        &mut self,
        address: Address,
        delay: Duration,
        type_name: TypeSpec<T>,
        value: &T,
    ) {
        let serialized = value.serialize(type_name.typename.to_string());
        self.delayed_invocations
            .push((address, delay, type_name.typename.to_string(), serialized));
    }

    /// Sends a message to the egress identifier by the `EgressIdentifier`.
    // todo: constrain it with Serializable
    pub fn egress<T: Serializable>(&mut self, identifier: EgressIdentifier, type_name: TypeSpec<T>, value: &T) {
        let serialized = value.serialize(type_name.typename.to_string());
        self.egress_messages
            .push((identifier, type_name.typename.to_string(), serialized));
    }

    /// Deletes the state kept under the given name.
    pub fn delete_state<T: Serializable>(&mut self, value_spec: ValueSpec<T>) {
        self.state_updates
            .push(StateUpdate::Delete(value_spec.into()));
    }

    /// Updates the state stored under the given name to the given value.
    pub fn update_state<T: Serializable>(&mut self, value_spec: ValueSpec<T>, value: &T) {
        let serialized = value.serialize(value_spec.typename.to_string());
        log::debug!("-- drey: updated state: {:?}", serialized);
        self.state_updates
            .push(StateUpdate::Update(value_spec.into(), serialized));
    }
}
