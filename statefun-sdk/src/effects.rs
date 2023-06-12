use crate::Address;
use crate::EgressIdentifier;
use crate::StateUpdate;
use crate::ValueSpec;
use crate::ValueSpecBase;
use protobuf::well_known_types::Any;
use protobuf::Message;
use std::time::Duration;

/// Effects (or side effects) of a stateful function invocation.
///
/// This can be used to:
///  - send messages ourselves or other stateful functions
///  - send messages to an egress
///  - update the state of this stateful function, which will be available on future invocations
#[derive(Default, Debug)]
pub struct Effects {
    pub(crate) invocations: Vec<(Address, String, Any)>,
    pub(crate) delayed_invocations: Vec<(Address, Duration, String, Any)>,
    pub(crate) egress_messages: Vec<(EgressIdentifier, String, Any)>,
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
    pub fn send<M: Message>(&mut self, address: Address, value_spec: ValueSpecBase, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.invocations
            .push((address, value_spec.typename, packed_message));
    }

    /// Sends a message to the stateful function identified by the address after a delay.
    pub fn send_after<M: Message>(
        &mut self,
        address: Address,
        delay: Duration,
        value_spec: ValueSpecBase,
        message: M,
    ) {
        let packed_message = Any::pack(&message).unwrap();
        self.delayed_invocations
            .push((address, delay, value_spec.typename, packed_message));
    }

    /// Sends a message to the egress identifier by the `EgressIdentifier`.
    pub fn egress<M: Message>(
        &mut self,
        identifier: EgressIdentifier,
        value_spec: ValueSpecBase,
        message: M,
    ) {
        let packed_message = Any::pack(&message).unwrap();
        self.egress_messages
            .push((identifier, value_spec.typename, packed_message));
    }

    /// Deletes the state kept under the given name.
    pub fn delete_state(&mut self, value_spec: ValueSpecBase) {
        self.state_updates.push(StateUpdate::Delete(value_spec));
    }

    /// Updates the state stored under the given name to the given value.
    pub fn update_state<T>(&mut self, value_spec: ValueSpec<T>, value: &T) {
        let serialized = (value_spec.serializer)(value, value_spec.typename.to_string());
        log::debug!("-- drey: updated state: {:?}", serialized);
        self.state_updates
            .push(StateUpdate::Update(value_spec.into(), serialized));
    }
}
