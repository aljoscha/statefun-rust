//! A bridge between the Protobuf world and the world of the Rust SDK. For use by `Transports`.
use std::collections::HashMap;

use protobuf::SingularPtrField;

use statefun_proto::request_reply::FromFunction;
use statefun_proto::request_reply::FromFunction_DelayedInvocation;
use statefun_proto::request_reply::FromFunction_EgressMessage;
use statefun_proto::request_reply::FromFunction_ExpirationSpec;
use statefun_proto::request_reply::FromFunction_ExpirationSpec_ExpireMode;
use statefun_proto::request_reply::FromFunction_IncompleteInvocationContext;
use statefun_proto::request_reply::FromFunction_Invocation;
use statefun_proto::request_reply::FromFunction_InvocationResponse;
use statefun_proto::request_reply::FromFunction_PersistedValueMutation;
use statefun_proto::request_reply::FromFunction_PersistedValueMutation_MutationType;
use statefun_proto::request_reply::FromFunction_PersistedValueSpec;
use statefun_proto::request_reply::ToFunction;
use statefun_proto::request_reply::ToFunction_PersistedValue;
use statefun_proto::request_reply::TypedValue;

use crate::function_registry::FunctionRegistry;
use crate::{Address, Context, Expiration, ExpirationType, Message, DelayedInvocation, EgressIdentifier, InvocationError, StateUpdate, ValueSpecBase};

/// An invokable that takes protobuf `ToFunction` as argument and returns a protobuf `FromFunction`.
pub trait InvocationBridge {
    fn invoke_from_proto(&self, to_function: ToFunction) -> Result<FromFunction, InvocationError>;
}

impl InvocationBridge for FunctionRegistry {
    fn invoke_from_proto(
        &self,
        mut to_function: ToFunction,
    ) -> Result<FromFunction, InvocationError> {
        let mut batch_request = to_function.take_invocation();
        log::debug!(
            "FunctionRegistry: processing batch request {:#?}",
            batch_request
        );

        let self_address = batch_request.take_target();
        let persisted_values = batch_request.take_state();
        let mut persisted_values = parse_persisted_values(&persisted_values);

        // we maintain a map of state updates that we update after every invocation. We maintain
        // this to be able to send back coalesced state updates to the statefun runtime but we
        // also need to update persisted_values so that subsequent invocations also "see" state
        // updates
        let mut coalesced_state_updates: HashMap<ValueSpecBase, StateUpdate> = HashMap::new();

        let mut invocation_response = FromFunction_InvocationResponse::new();

        for mut invocation in batch_request.take_invocations().into_iter() {
            let caller_address = invocation.take_caller();
            let argument = Message::new(invocation.take_argument());
            let context = Context::new(&persisted_values, &self_address, &caller_address);

            let effects = match self.invoke(context.self_address().function_type, context, argument)
            {
                Ok(effects) => effects,
                Err(e) => match &e {
                    InvocationError::MissingStates(state_collection) => {
                        let mut incomplete_context =
                            FromFunction_IncompleteInvocationContext::new();

                        for value_spec in state_collection.states.iter() {
                            let mut expiration_spec = FromFunction_ExpirationSpec::new();

                            match &value_spec.expiration.expiration_type {
                                Some(expiration_type) => {
                                    expiration_spec.mode = match expiration_type {
                                        ExpirationType::AfterInvoke => FromFunction_ExpirationSpec_ExpireMode::AFTER_INVOKE,
                                        ExpirationType::AfterWrite => FromFunction_ExpirationSpec_ExpireMode::AFTER_WRITE,
                                    };

                                    expiration_spec.expire_after_millis = value_spec.expiration.time_to_live.as_millis() as i64;
                                    log::debug!("-- drey Set expiration time as {:?}", expiration_spec.expire_after_millis);
                                }
                                None => {
                                    expiration_spec.mode = FromFunction_ExpirationSpec_ExpireMode::NONE;
                                    expiration_spec.expire_after_millis = 0;
                                }
                            }

                            let mut persisted_value_spec = FromFunction_PersistedValueSpec::new();
                            persisted_value_spec.expiration_spec =
                                SingularPtrField::some(expiration_spec);

                            persisted_value_spec.state_name = value_spec.name.clone();
                            persisted_value_spec.type_typename = value_spec.typename.clone();

                            incomplete_context.missing_values.push(persisted_value_spec);
                        }

                        let mut from_function = FromFunction::new();
                        from_function.set_incomplete_invocation_context(incomplete_context);

                        return Ok(from_function);
                    }
                    _ => return Err(e),
                },
            };

            serialize_invocation_messages(&mut invocation_response, effects.invocations);
            serialize_delayed_invocation_messages(
                &mut invocation_response,
                effects.delayed_invocations,
            );
            serialize_cancelled_delayed_messages(
                &mut invocation_response,
                effects.cancelled_delayed_invocations,
            );
            serialize_egress_messages(&mut invocation_response, effects.egress_messages);
            update_state(
                &mut persisted_values,
                &mut coalesced_state_updates,
                effects.state_updates,
            );
        }

        let state_values = coalesced_state_updates.drain().map(|(_key, value)| value);
        serialize_state_updates(&mut invocation_response, state_values)?;

        let mut from_function = FromFunction::new();
        from_function.set_invocation_result(invocation_response);

        Ok(from_function)
    }
}

fn to_typed_value(typename: String, value: Vec<u8>) -> TypedValue {
    let mut res = TypedValue::new();
    res.set_typename(typename);
    res.set_has_value(true);
    res.set_value(value);
    res
}

fn parse_persisted_values(
    persisted_values: &[ToFunction_PersistedValue],
) -> HashMap<ValueSpecBase, Vec<u8>> {
    let mut result = HashMap::new();
    for persisted_value in persisted_values {
        result.insert(
            ValueSpecBase::new(
                persisted_value.get_state_name(),
                persisted_value.get_state_value().get_typename(),
                Expiration::never(),  // note: Flink never gives this info back to us,
                                      // so we have to be careful to omit it when doing
                                      // lookups later in the Context
            ),
            persisted_value.get_state_value().get_value().to_vec(),
        );
    }
    result
}

fn update_state(
    persisted_state: &mut HashMap<ValueSpecBase, Vec<u8>>,
    coalesced_state: &mut HashMap<ValueSpecBase, StateUpdate>,
    state_updates: Vec<StateUpdate>,
) {
    for state_update in state_updates {
        match state_update {
            StateUpdate::Delete(value_spec) => {
                // Note: we don't remove persisted state, we only remove the value.
                // This mimics what Flink does, and ensures that batch requests to the same function
                // work correctly. For example, a 2x batch request to the same function where the
                // function calls `update("mystate"); delete("mystate")` should result in a response
                // to apache requesting `mystate` value to be removed.
                //
                // If we removed this `persisted_state` then the 2nd invocation within the batch
                // would cause the function to respond with `incomplete_invocation_context`,
                // eventually causing Flink to request the batched request to the function,
                // where again the 2nd invocation in the function will respond with
                // `incomplete_invocation_context`.
                //
                // This leads to an infinite loop.
                //
                // Instead the state has to be marked as cleared out, but the key is never deleted.
                // Remember that this key is part of the function's registered state signature.
                persisted_state.insert(value_spec.clone(), vec![]);
                coalesced_state.insert(value_spec.clone(), StateUpdate::Delete(value_spec.clone()));
            }
            StateUpdate::Update(value_spec, state) => {
                persisted_state.insert(value_spec.clone(), state.clone());
                coalesced_state.insert(
                    value_spec.clone(),
                    StateUpdate::Update(value_spec.clone(), state.clone()),
                );
            }
        }
    }
}

fn serialize_invocation_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    invocation_messages: Vec<(Address, String, Vec<u8>)>,
) {
    for invocation_message in invocation_messages {
        let mut proto_invocation_message = FromFunction_Invocation::new();
        proto_invocation_message.set_target(invocation_message.0.into_proto());
        let typed_value = to_typed_value(invocation_message.1, invocation_message.2);
        proto_invocation_message.set_argument(typed_value);
        invocation_response
            .outgoing_messages
            .push(proto_invocation_message);
    }
}

fn serialize_delayed_invocation_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    delayed_invocations: Vec<DelayedInvocation>,
) {
    for invocation_message in delayed_invocations {
        let mut proto_invocation_message = FromFunction_DelayedInvocation::new();
        proto_invocation_message.set_target(invocation_message.address.into_proto());
        proto_invocation_message.set_delay_in_ms(invocation_message.delay.as_millis() as i64);
        proto_invocation_message.set_cancellation_token(invocation_message.cancellation_token);
        let typed_value = to_typed_value(invocation_message.typename, invocation_message.bytes);
        proto_invocation_message.set_argument(typed_value);
        invocation_response
            .delayed_invocations
            .push(proto_invocation_message);
    }
}

fn serialize_cancelled_delayed_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    cancelled_delayed_invocations: Vec<String>,
) {
    for cancel_invocation_tokens in cancelled_delayed_invocations {
        let mut proto_invocation_message = FromFunction_DelayedInvocation::new();
        proto_invocation_message.set_is_cancellation_request(true);
        proto_invocation_message.set_cancellation_token(cancel_invocation_tokens);
        invocation_response
            .delayed_invocations
            .push(proto_invocation_message);
    }
}

fn serialize_egress_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    egress_messages: Vec<(EgressIdentifier, String, Vec<u8>)>,
) {
    for egress_message in egress_messages {
        let mut proto_egress_message = FromFunction_EgressMessage::new();
        proto_egress_message.set_egress_namespace(egress_message.0.namespace);
        proto_egress_message.set_egress_type(egress_message.0.name);
        let typed_value = to_typed_value(egress_message.1, egress_message.2);
        proto_egress_message.set_argument(typed_value);
        invocation_response
            .outgoing_egresses
            .push(proto_egress_message);
    }
}

fn serialize_state_updates<T>(
    invocation_response: &mut FromFunction_InvocationResponse,
    state_updates: T,
) -> Result<(), InvocationError>
where
    T: IntoIterator<Item = StateUpdate>,
{
    for state_update in state_updates {
        match state_update {
            StateUpdate::Delete(value_spec) => {
                let mut proto_state_update = FromFunction_PersistedValueMutation::new();
                proto_state_update.set_state_name(value_spec.name);
                // Note: DELETE is the default enum and will not be serialized over the wire,
                // this is an optimization in Protobuf: https://stackoverflow.com/a/71207894
                proto_state_update
                    .set_mutation_type(FromFunction_PersistedValueMutation_MutationType::DELETE);
                invocation_response.state_mutations.push(proto_state_update);
            }

            StateUpdate::Update(value_spec, state) => {
                let mut proto_state_update = FromFunction_PersistedValueMutation::new();
                proto_state_update.set_state_name(value_spec.name);

                proto_state_update.set_state_value(to_typed_value(value_spec.typename, state));
                proto_state_update
                    .set_mutation_type(FromFunction_PersistedValueMutation_MutationType::MODIFY);
                invocation_response.state_mutations.push(proto_state_update);
            }
        }
    }
    Ok(())
}

// /// Unpacks the given state, which is expected to be a serialized `Any<T>`.
// fn unpack_state<T: Message>(value_spec: ValueSpecBase, packed_state: &Any) -> Option<T> {
//     // let packed_state: Any =
//     //     protobuf::parse_from_bytes(serialized_state).expect("Could not deserialize state.");

//     log::debug!("Packed state for {:?}: {:?}", value_spec, packed_state);

//     let unpacked_state: Option<T> = packed_state
//         .unpack()
//         .expect("Could not unpack state from Any.");

//     unpacked_state
// }

#[cfg(test)]
mod tests {
    use core::time::Duration;
    // use protobuf::well_known_types::Any;
    // use protobuf::Message;

    // use protobuf::well_known_types::{Int32Value};
    use protobuf::RepeatedField;

    use statefun_proto::request_reply::FromFunction_DelayedInvocation;
    use statefun_proto::request_reply::FromFunction_EgressMessage;
    use statefun_proto::request_reply::FromFunction_Invocation;
    // use statefun_proto::request_reply::FromFunction_PersistedValueMutation;
    // use statefun_proto::request_reply::FromFunction_PersistedValueMutation_MutationType;
    use statefun_proto::request_reply::ToFunction;
    use statefun_proto::request_reply::ToFunction_Invocation;
    use statefun_proto::request_reply::ToFunction_InvocationBatchRequest;
    use statefun_proto::request_reply::ToFunction_PersistedValue;

    use crate::invocation_bridge::{InvocationBridge};
    use crate::FunctionRegistry;
    use crate::*;

    fn foo_state() -> ValueSpec<i32> { ValueSpec::<i32>::new("foo", Expiration::never()) }
    fn bar_state() -> ValueSpec<i32> { ValueSpec::<i32>::new("bar", Expiration::never()) }

    const MESSAGE1: &str = "fli";
    const MESSAGE2: &str = "fla";
    const MESSAGE3: &str = "flu";

    // Verifies that all possible fields in a ToFunction are accessible in a function
    #[test]
    fn forward_to_function() -> anyhow::Result<()> {
        let mut registry = FunctionRegistry::new();

        registry.register_fn(function_type(), vec![foo_state().into(), bar_state().into()],
                             |context, message: Message| {
            assert_eq!(context.self_address(), self_address());
            assert_eq!(context.caller_address(), caller_address());
            assert_eq!(
                context
                    .get_state::<i32>(foo_state())
                    .expect("State not here.")
                    .unwrap(),
                42
            );
            assert_eq!(
                context
                    .get_state::<i32>(bar_state())
                    .expect("State not here.")
                    .unwrap(),
                84
            );

            let string_message = message.get::<String>().unwrap();
            let mut effects = Effects::new();

            // the test checks against this message to ensure that the function was invoked
            // and all the asserts above were executed
            effects.send(
                self_address(),
                &string_message,
            )
            .unwrap();

            effects
        });

        // request
        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        // response
        let mut invocation_response = from_function.take_invocation_result();
        let mut outgoing = invocation_response.take_outgoing_messages();

        assert_invocation(outgoing.remove(0), self_address(), MESSAGE1.to_string());
        assert_invocation(outgoing.remove(0), self_address(), MESSAGE2.to_string());
        assert_invocation(outgoing.remove(0), self_address(), MESSAGE3.to_string());

        Ok(())
    }

    // Verifies that messages are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_messages_from_function() -> anyhow::Result<()> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), vec![foo_state().into(), bar_state().into()],
                             |_context, message: Message| {
            let string_message = message.get::<String>().unwrap();
            let mut effects = Effects::new();

            effects.send(
                self_address(),
                &string_message,
            )
            .unwrap();

            effects
        });

        // request
        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        // response
        let mut invocation_response = from_function.take_invocation_result();
        let mut outgoing = invocation_response.take_outgoing_messages();

        assert_invocation(outgoing.remove(0), self_address(), MESSAGE1.to_string());
        assert_invocation(outgoing.remove(0), self_address(), MESSAGE2.to_string());
        assert_invocation(outgoing.remove(0), self_address(), MESSAGE3.to_string());

        Ok(())
    }

    // Verifies that delayed messages are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_delayed_messages_from_function() -> anyhow::Result<()> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), vec![], |_context, message| {
            let string_message = message.get::<String>().unwrap();
            let mut effects = Effects::new();

            effects.send_after(caller_address(), Duration::from_secs(5), "cancel-token".to_string(),
                &string_message).unwrap();

            effects
        });

        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_response = from_function.take_invocation_result();
        let mut delayed = invocation_response.take_delayed_invocations();

        assert_delayed_invocation(
            delayed.remove(0),
            caller_address(),
            5000,
            false,
            "cancel-token".to_string(),
            MESSAGE1.to_string(),
        );
        assert_delayed_invocation(
            delayed.remove(0),
            caller_address(),
            5000,
            false,
            "cancel-token".to_string(),
            MESSAGE2.to_string(),
        );
        assert_delayed_invocation(
            delayed.remove(0),
            caller_address(),
            5000,
            false,
            "cancel-token".to_string(),
            MESSAGE3.to_string(),
        );

        Ok(())
    }

    // Verifies that egresses are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_egresses_from_function() -> anyhow::Result<()> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), vec![], |_context, _message| {
            let mut effects = Effects::new();

            effects.egress(
                EgressIdentifier::new("namespace", "name"),
                &"egress".to_string(),
            ).unwrap();

            effects
        });

        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_response = from_function.take_invocation_result();
        let mut egresses = invocation_response.take_outgoing_egresses();

        assert_egress(
            egresses.remove(0),
            "namespace",
            "name",
            "egress".to_string(),
        );
        assert_egress(
            egresses.remove(0),
            "namespace",
            "name",
            "egress".to_string(),
        );
        assert_egress(
            egresses.remove(0),
            "namespace",
            "name",
            "egress".to_string(),
        );

        Ok(())
    }

    // // Verifies that state mutations are correctly forwarded to the Protobuf FromFunction
    // #[test]
    // fn forward_state_mutations_from_function() -> anyhow::Result<()> {
    //     let mut registry = FunctionRegistry::new();
    //     registry.register_fn(function_type(), |_context, _message: String| {
    //         let mut effects = Effects::new();

    //         effects.update_state(bar_state, &i32_value(84));
    //         effects.delete_state(foo_state);

    //         effects
    //     });

    //     let to_function = complete_to_function();
    //     let mut from_function = registry.invoke_from_proto(to_function)?;

    //     let mut invocation_response = from_function.take_invocation_result();
    //     let state_mutations = invocation_response.take_state_mutations();

    //     let state_map = to_state_map(state_mutations);
    //     assert_eq!(state_map.len(), 2);

    //     let bar_state = state_map.get(bar_state).unwrap();
    //     let foo_state = state_map.get(foo_state).unwrap();

    //     // state updates are coalesced
    //     assert_state_update(bar_state, bar_state, i32_value(84));
    //     assert_state_delete(foo_state, foo_state);

    //     Ok(())
    // }

    // fn to_state_map(
    //     state_mutations: RepeatedField<FromFunction_PersistedValueMutation>,
    // ) -> HashMap<String, FromFunction_PersistedValueMutation> {
    //     let mut state_mutations_map = HashMap::new();
    //     for state_mutation in state_mutations.into_iter() {
    //         state_mutations_map.insert(state_mutation.get_state_name().to_string(), state_mutation);
    //     }
    //     state_mutations_map
    // }

    // // Verifies that state mutations are correctly forwarded to the Protobuf FromFunction
    // #[test]
    // fn state_mutations_available_in_subsequent_invocations() -> anyhow::Result<()> {
    //     let mut registry = FunctionRegistry::new();
    //     registry.register_fn(function_type(), |context, _message: String| {
    //         let state: Int32Value = context.get_state(bar_state).unwrap();

    //         let mut effects = Effects::new();
    //         effects.update_state(bar_state, &i32_value(state.get_value() + 1));
    //         effects.delete_state(foo_state);

    //         effects
    //     });

    //     let to_function = complete_to_function();
    //     let mut from_function = registry.invoke_from_proto(to_function)?;

    //     let mut invocation_response = from_function.take_invocation_result();
    //     let state_mutations = invocation_response.take_state_mutations();

    //     let state_map = to_state_map(state_mutations);
    //     assert_eq!(state_map.len(), 2);

    //     let bar_state = state_map.get(bar_state).unwrap();
    //     let foo_state = state_map.get(foo_state).unwrap();

    //     // state updates are coalesced
    //     assert_state_update(bar_state, bar_state, i32_value(3));
    //     assert_state_delete(foo_state, foo_state);

    //     Ok(())
    // }

    fn assert_invocation(
        invocation: FromFunction_Invocation,
        expected_address: Address,
        expected_message: String,
    ) {
        assert_eq!(
            Address::from_proto(invocation.get_target()),
            expected_address
        );

        assert_eq!(
            String::deserialize(String::get_typename().to_string(),
                &invocation.get_argument().get_value().to_vec()).unwrap(),
            expected_message
        );
    }

    fn assert_delayed_invocation(
        invocation: FromFunction_DelayedInvocation,
        expected_address: Address,
        expected_delay: i64,
        is_cancellation: bool,
        cancellation_token: String,
        expected_message: String,
    ) {
        assert_eq!(
            Address::from_proto(invocation.get_target()),
            expected_address
        );
        assert_eq!(invocation.get_delay_in_ms(), expected_delay);
        assert_eq!(
            invocation.get_is_cancellation_request(),
            is_cancellation);

        assert_eq!(
            invocation.get_cancellation_token(),
            cancellation_token);
        assert_eq!(
            String::deserialize(String::get_typename().to_string(),
                &invocation.get_argument().get_value().to_vec()).unwrap(),
            expected_message
        );
    }

    fn assert_egress(
        egress: FromFunction_EgressMessage,
        expected_namespace: &str,
        expected_name: &str,
        expected_message: String,
    ) {
        assert_eq!(egress.get_egress_namespace(), expected_namespace);
        assert_eq!(egress.get_egress_type(), expected_name);
        assert_eq!(
            String::deserialize(String::get_typename().to_string(),
                &egress.get_argument().get_value().to_vec()).unwrap(),
            expected_message
        );
    }

    // fn assert_state_update<T: Message + PartialEq>(
    //     state_mutation: &FromFunction_PersistedValueMutation,
    //     expected_name: &str,
    //     expected_value: T,
    // ) {
    //     assert_eq!(
    //         state_mutation.get_mutation_type(),
    //         FromFunction_PersistedValueMutation_MutationType::MODIFY
    //     );
    //     assert_eq!(state_mutation.get_state_name(), expected_name);
    //     let packed_state: Any = deserialize_state(state_mutation.get_state_value());
    //     let unpacked_state_value: Option<T> = unpack_state(expected_name, &packed_state);
    //     assert_eq!(unpacked_state_value.unwrap(), expected_value)
    // }

    // fn assert_state_delete(
    //     state_mutation: &FromFunction_PersistedValueMutation,
    //     expected_name: &str,
    // ) {
    //     assert_eq!(
    //         state_mutation.get_mutation_type(),
    //         FromFunction_PersistedValueMutation_MutationType::DELETE
    //     );
    //     assert_eq!(state_mutation.get_state_name(), expected_name);
    // }

    /// Creates a complete Protobuf ToFunction that contains every possible field/type, including
    /// multiple invocations to test batching behaviour.
    fn complete_to_function() -> ToFunction {
        let mut to_function = ToFunction::new();
        let invocation_batch = complete_batch_request();
        to_function.set_invocation(invocation_batch);
        to_function
    }

    fn complete_batch_request() -> ToFunction_InvocationBatchRequest {
        let mut invocation_batch = ToFunction_InvocationBatchRequest::new();

        invocation_batch.set_target(self_address().into_proto());
        invocation_batch.set_state(states());
        invocation_batch.set_invocations(invocations());

        invocation_batch
    }

    fn function_type() -> FunctionType {
        FunctionType::new("namespace", "foo")
    }

    fn self_address() -> Address {
        Address::new(function_type(), "self")
    }

    fn caller_address() -> Address {
        Address::new(function_type(), "caller")
    }

    fn states() -> RepeatedField<ToFunction_PersistedValue> {
        let mut states = RepeatedField::new();

        // todo: this needs to be state name
        states.push(state(foo_state().into(), 42));
        states.push(state(bar_state().into(), 84));

        states
    }

    fn state(value_spec: ValueSpecBase, value: i32) -> ToFunction_PersistedValue {
        let mut state = ToFunction_PersistedValue::new();

        let mut typed_value = TypedValue::new();
        typed_value.set_typename(value_spec.typename);
        typed_value.set_has_value(true);
        typed_value.set_value(value.serialize(String::get_typename().to_string()).unwrap());

        state.set_state_name(value_spec.name);
        state.set_state_value(typed_value);

        state
    }

    // It's important to create multiple invocations to test whether state updates can be "seen"
    // by later invocations in a batch.
    fn invocations() -> RepeatedField<ToFunction_Invocation> {
        let mut invocations = RepeatedField::new();

        invocations.push(invocation(caller_address(), MESSAGE1.to_string()));
        invocations.push(invocation(caller_address(), MESSAGE2.to_string()));
        invocations.push(invocation(caller_address(), MESSAGE3.to_string()));

        invocations
    }

    fn invocation(caller: Address, argument: String) -> ToFunction_Invocation {
        let mut invocation = ToFunction_Invocation::new();

        let mut typed_value = TypedValue::new();
        typed_value.set_typename(String::get_typename().to_string());
        typed_value.set_has_value(true);
        typed_value.set_value(argument.serialize(String::get_typename().to_string()).unwrap());

        // let message = string_value(argument);
        // let packed_argument = Any::pack(&message).unwrap();
        invocation.set_caller(caller.into_proto());
        invocation.set_argument(typed_value);

        invocation
    }

    // fn string_value(value: &str) -> String {
    //     // String (and generated code from protobuf in general) is not very ergonomic...
    //     let mut result = String::new();
    //     result.set_value(value.to_owned());
    //     result
    // }

    // fn i32_value(value: i32) -> Int32Value {
    //     let mut result = Int32Value::new();
    //     result.set_value(value);
    //     result
    // }

    // fn unpack_any<M: Message>(any: &Any) -> M {
    //     any.unpack()
    //         .expect("Could not unwrap Result")
    //         .expect("Could not unwrap Option.")
    // }
}
