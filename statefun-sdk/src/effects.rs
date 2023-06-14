use crate::Address;
use crate::DelayedInvocation;
use crate::EgressIdentifier;
use crate::Serializable;
use crate::StateUpdate;
use crate::TypeSpec;
use crate::ValueSpec;
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
    pub(crate) delayed_invocations: Vec<DelayedInvocation>,
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
    pub fn send<T: Serializable<T>>(
        &mut self,
        address: Address,
        type_name: TypeSpec<T>,
        value: &T,
    ) -> Result<(), String> {
        let serialized = value.serialize(type_name.typename.to_string())?;
        Ok(self
            .invocations
            .push((address, type_name.typename.to_string(), serialized)))
    }

    /// Sends a message to the stateful function identified by the address after a delay.
    pub fn send_after<T: Serializable<T>>(
        &mut self,
        address: Address,
        delay: Duration,
        cancellation_token: String,
        type_name: TypeSpec<T>,
        value: &T,
    ) -> Result<(), String> {
        let serialized = value.serialize(type_name.typename.to_string())?;
        Ok(self.delayed_invocations.push(DelayedInvocation::new(
            address,
            delay,
            cancellation_token,
            type_name.typename.to_string(),
            serialized,
        )))
    }

    /// Sends a message to the egress identifier by the `EgressIdentifier`.
    pub fn egress<T: Serializable<T>>(
        &mut self,
        identifier: EgressIdentifier,
        type_name: TypeSpec<T>,
        value: &T,
    ) -> Result<(), String> {
        let serialized = value.serialize(type_name.typename.to_string())?;
        Ok(self
            .egress_messages
            .push((identifier, type_name.typename.to_string(), serialized)))
    }

    /// Deletes the state kept under the given name.
    pub fn delete_state<T: Serializable<T>>(&mut self, value_spec: ValueSpec<T>) {
        self.state_updates
            .push(StateUpdate::Delete(value_spec.into()));
    }

    /// Updates the state stored under the given name to the given value.
    pub fn update_state<T: Serializable<T>>(
        &mut self,
        value_spec: ValueSpec<T>,
        value: &T,
    ) -> Result<(), String> {
        let serialized = value.serialize(value_spec.spec.typename.to_string())?;
        Ok(self
            .state_updates
            .push(StateUpdate::Update(value_spec.into(), serialized)))
    }
}
