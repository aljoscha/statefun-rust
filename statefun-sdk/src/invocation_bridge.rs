///! A bridge between the Protobuf world and the world of the Rust SDK. For use by `Transports`.
use std::collections::HashMap;
use std::time::Duration;

use protobuf::well_known_types::Any;
use protobuf::Message;

use statefun_proto::http_function::FromFunction;
use statefun_proto::http_function::FromFunction_DelayedInvocation;
use statefun_proto::http_function::FromFunction_EgressMessage;
use statefun_proto::http_function::FromFunction_Invocation;
use statefun_proto::http_function::FromFunction_InvocationResponse;
use statefun_proto::http_function::FromFunction_PersistedValueMutation;
use statefun_proto::http_function::FromFunction_PersistedValueMutation_MutationType;
use statefun_proto::http_function::ToFunction;
use statefun_proto::http_function::ToFunction_PersistedValue;

use crate::function_registry::FunctionRegistry;
use crate::{Address, Context, EgressIdentifier, StateUpdate};

/// An invokable that takes protobuf `ToFunction` as argument and returns a protobuf `FromFunction`.
pub trait InvocationBridge {
    fn invoke_from_proto(&self, to_function: ToFunction) -> Result<FromFunction, failure::Error>;
}

impl InvocationBridge for FunctionRegistry {
    fn invoke_from_proto(
        &self,
        mut to_function: ToFunction,
    ) -> Result<FromFunction, failure::Error> {
        let mut batch_request = to_function.take_invocation();
        log::debug!(
            "FunctionRegistry: processing batch request {:#?}",
            batch_request
        );

        let self_address = batch_request.take_target();
        let persisted_values_proto = batch_request.take_state();
        let persisted_values = parse_persisted_values(&persisted_values_proto);

        let mut invocation_respose = FromFunction_InvocationResponse::new();

        for mut invocation in batch_request.take_invocations().into_iter() {
            let caller_address = invocation.take_caller();
            let argument = invocation.take_argument();
            let context = Context::new(&persisted_values, &self_address, &caller_address);

            let effects = self.invoke(context.self_address().function_type, context, argument)?;

            serialize_invocation_messages(&mut invocation_respose, effects.invocations);
            serialize_delayed_invocation_messages(
                &mut invocation_respose,
                effects.delayed_invocations,
            );
            serialize_egress_messages(&mut invocation_respose, effects.egress_messages);
            serialize_state_updates(&mut invocation_respose, effects.state_updates)?;
        }

        let mut from_function = FromFunction::new();
        from_function.set_invocation_result(invocation_respose);

        Ok(from_function)
    }
}

fn parse_persisted_values(persisted_values: &[ToFunction_PersistedValue]) -> HashMap<&str, &[u8]> {
    let mut result = HashMap::new();
    for persisted_value in persisted_values {
        result.insert(
            persisted_value.get_state_name(),
            persisted_value.get_state_value(),
        );
    }
    result
}

fn serialize_invocation_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    invocation_messages: Vec<(Address, Any)>,
) {
    for invocation_message in invocation_messages {
        let mut proto_invocation_message = FromFunction_Invocation::new();
        proto_invocation_message.set_target(invocation_message.0.into_proto());
        proto_invocation_message.set_argument(invocation_message.1);
        invocation_response
            .outgoing_messages
            .push(proto_invocation_message);
    }
}

fn serialize_delayed_invocation_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    delayed_invocation_messages: Vec<(Address, Duration, Any)>,
) {
    for invocation_message in delayed_invocation_messages {
        let mut proto_invocation_message = FromFunction_DelayedInvocation::new();
        proto_invocation_message.set_target(invocation_message.0.into_proto());
        proto_invocation_message.set_delay_in_ms(invocation_message.1.as_millis() as i64);
        proto_invocation_message.set_argument(invocation_message.2);
        invocation_response
            .delayed_invocations
            .push(proto_invocation_message);
    }
}

fn serialize_egress_messages(
    invocation_response: &mut FromFunction_InvocationResponse,
    egress_messages: Vec<(EgressIdentifier, Any)>,
) {
    for egress_message in egress_messages {
        let mut proto_egress_message = FromFunction_EgressMessage::new();
        proto_egress_message.set_egress_namespace(egress_message.0.namespace);
        proto_egress_message.set_egress_type(egress_message.0.name);
        proto_egress_message.set_argument(egress_message.1);
        invocation_response
            .outgoing_egresses
            .push(proto_egress_message);
    }
}

fn serialize_state_updates(
    invocation_response: &mut FromFunction_InvocationResponse,
    state_updates: Vec<StateUpdate>,
) -> Result<(), failure::Error> {
    for state_update in state_updates {
        match state_update {
            StateUpdate::Delete(name) => {
                let mut proto_state_update = FromFunction_PersistedValueMutation::new();
                proto_state_update.set_state_name(name);
                proto_state_update
                    .set_mutation_type(FromFunction_PersistedValueMutation_MutationType::DELETE);
                invocation_response.state_mutations.push(proto_state_update);
            }
            StateUpdate::Update(name, state) => {
                let mut proto_state_update = FromFunction_PersistedValueMutation::new();
                proto_state_update.set_state_name(name);
                proto_state_update.set_state_value(state.write_to_bytes()?);
                proto_state_update
                    .set_mutation_type(FromFunction_PersistedValueMutation_MutationType::MODIFY);
                invocation_response.state_mutations.push(proto_state_update);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use protobuf::well_known_types::{Int32Value, StringValue};
    use protobuf::RepeatedField;

    use statefun_proto::http_function::FromFunction_DelayedInvocation;
    use statefun_proto::http_function::FromFunction_EgressMessage;
    use statefun_proto::http_function::FromFunction_Invocation;
    use statefun_proto::http_function::FromFunction_PersistedValueMutation;
    use statefun_proto::http_function::FromFunction_PersistedValueMutation_MutationType;
    use statefun_proto::http_function::ToFunction;
    use statefun_proto::http_function::ToFunction_Invocation;
    use statefun_proto::http_function::ToFunction_InvocationBatchRequest;
    use statefun_proto::http_function::ToFunction_PersistedValue;

    use crate::invocation_bridge::InvocationBridge;
    use crate::FunctionRegistry;
    use crate::*;

    const FOO_STATE: &str = "foo";
    const BAR_STATE: &str = "bar";
    const MESSAGE1: &str = "fli";
    const MESSAGE2: &str = "fla";
    const MESSAGE3: &str = "flu";

    // Verifies that all possible fields in a ToFunction are accessible in a function
    #[test]
    fn forward_to_function() -> Result<(), failure::Error> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), |context, message: StringValue| {
            assert_eq!(context.self_address(), self_address());
            assert_eq!(context.caller_address(), caller_address());
            assert_eq!(
                context
                    .get_state::<Int32Value>(FOO_STATE)
                    .expect("State not here."),
                i32_value(0)
            );
            assert_eq!(
                context
                    .get_state::<Int32Value>(BAR_STATE)
                    .expect("State not here."),
                i32_value(0)
            );

            let mut effects = Effects::new();

            // the test checks against this message to ensure that the function was invoked
            // and all the asserts above were executed
            effects.send(self_address(), message);

            effects
        });

        let to_function = complete_to_function();

        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_respose = from_function.take_invocation_result();
        let mut outgoing = invocation_respose.take_outgoing_messages();

        assert_invocation(outgoing.remove(0), self_address(), string_value(MESSAGE1));
        assert_invocation(outgoing.remove(0), self_address(), string_value(MESSAGE2));
        assert_invocation(outgoing.remove(0), self_address(), string_value(MESSAGE3));

        Ok(())
    }

    // Verifies that messages are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_messages_from_function() -> Result<(), failure::Error> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), |_context, message: StringValue| {
            let mut effects = Effects::new();

            effects.send(self_address(), message.clone());

            effects
        });

        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_respose = from_function.take_invocation_result();
        let mut outgoing = invocation_respose.take_outgoing_messages();

        assert_invocation(outgoing.remove(0), self_address(), string_value(MESSAGE1));
        assert_invocation(outgoing.remove(0), self_address(), string_value(MESSAGE2));
        assert_invocation(outgoing.remove(0), self_address(), string_value(MESSAGE3));

        Ok(())
    }

    // Verifies that delayed messages are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_delayed_messages_from_function() -> Result<(), failure::Error> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), |_context, message: StringValue| {
            let mut effects = Effects::new();

            effects.send_after(caller_address(), Duration::from_secs(5), message.clone());

            effects
        });

        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_respose = from_function.take_invocation_result();
        let mut delayed = invocation_respose.take_delayed_invocations();

        assert_delayed_invocation(
            delayed.remove(0),
            caller_address(),
            5000,
            string_value(MESSAGE1),
        );
        assert_delayed_invocation(
            delayed.remove(0),
            caller_address(),
            5000,
            string_value(MESSAGE2),
        );
        assert_delayed_invocation(
            delayed.remove(0),
            caller_address(),
            5000,
            string_value(MESSAGE3),
        );

        Ok(())
    }

    // Verifies that egresses are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_egresses_from_function() -> Result<(), failure::Error> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), |_context, _message: StringValue| {
            let mut effects = Effects::new();

            effects.egress(
                EgressIdentifier::new("namespace", "name"),
                string_value("egress"),
            );

            effects
        });

        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_respose = from_function.take_invocation_result();
        let mut egresses = invocation_respose.take_outgoing_egresses();

        assert_egress(
            egresses.remove(0),
            "namespace",
            "name",
            string_value("egress"),
        );
        assert_egress(
            egresses.remove(0),
            "namespace",
            "name",
            string_value("egress"),
        );
        assert_egress(
            egresses.remove(0),
            "namespace",
            "name",
            string_value("egress"),
        );

        Ok(())
    }

    // Verifies that state mutations are correctly forwarded to the Protobuf FromFunction
    #[test]
    fn forward_state_mutations_from_function() -> Result<(), failure::Error> {
        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type(), |_context, _message: StringValue| {
            let mut effects = Effects::new();

            effects.update_state(BAR_STATE, &i32_value(42));
            effects.delete_state(FOO_STATE);

            effects
        });

        let to_function = complete_to_function();
        let mut from_function = registry.invoke_from_proto(to_function)?;

        let mut invocation_respose = from_function.take_invocation_result();
        let mut state_mutations = invocation_respose.take_state_mutations();

        // TODO: This behaviour is incorrect. The processing logic must coalesce state updates
        // and state updates of one invocation in a batch must be visible to subsequent
        // invocations
        assert_state_update(state_mutations.remove(0), BAR_STATE, i32_value(42));
        assert_state_delete(state_mutations.remove(0), FOO_STATE);

        assert_state_update(state_mutations.remove(0), BAR_STATE, i32_value(42));
        assert_state_delete(state_mutations.remove(0), FOO_STATE);

        assert_state_update(state_mutations.remove(0), BAR_STATE, i32_value(42));
        assert_state_delete(state_mutations.remove(0), FOO_STATE);

        Ok(())
    }

    fn assert_invocation(
        invocation: FromFunction_Invocation,
        expected_address: Address,
        expected_message: StringValue,
    ) {
        assert_eq!(
            Address::from_proto(invocation.get_target()),
            expected_address
        );
        assert_eq!(
            unpack_any::<StringValue>(invocation.get_argument()),
            expected_message
        );
    }

    fn assert_delayed_invocation(
        invocation: FromFunction_DelayedInvocation,
        expected_address: Address,
        expected_delay: i64,
        expected_message: StringValue,
    ) {
        assert_eq!(
            Address::from_proto(invocation.get_target()),
            expected_address
        );
        assert_eq!(invocation.get_delay_in_ms(), expected_delay);
        assert_eq!(
            unpack_any::<StringValue>(invocation.get_argument()),
            expected_message
        );
    }

    fn assert_egress(
        egress: FromFunction_EgressMessage,
        expected_namespace: &str,
        expected_name: &str,
        expected_message: StringValue,
    ) {
        assert_eq!(egress.get_egress_namespace(), expected_namespace);
        assert_eq!(egress.get_egress_type(), expected_name);
        assert_eq!(
            unpack_any::<StringValue>(egress.get_argument()),
            expected_message
        );
    }

    fn assert_state_update<T: Message + PartialEq>(
        state_mutation: FromFunction_PersistedValueMutation,
        expected_name: &str,
        expected_value: T,
    ) {
        assert_eq!(
            state_mutation.get_mutation_type(),
            FromFunction_PersistedValueMutation_MutationType::MODIFY
        );
        assert_eq!(state_mutation.get_state_name(), expected_name);
        let unpacked_state_value: Option<T> =
            unpack_state(expected_name, state_mutation.get_state_value());
        assert_eq!(unpacked_state_value.unwrap(), expected_value)
    }

    fn assert_state_delete(
        state_mutation: FromFunction_PersistedValueMutation,
        expected_name: &str,
    ) {
        assert_eq!(
            state_mutation.get_mutation_type(),
            FromFunction_PersistedValueMutation_MutationType::DELETE
        );
        assert_eq!(state_mutation.get_state_name(), expected_name);
    }

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

        states.push(state(FOO_STATE.to_owned(), 0));
        states.push(state(BAR_STATE.to_owned(), 0));

        states
    }

    fn state(name: String, value: i32) -> ToFunction_PersistedValue {
        let mut state = ToFunction_PersistedValue::new();

        let state_proto_foo = i32_value(value);
        let any_foo = Any::pack(&state_proto_foo).unwrap();
        state.set_state_name(name);
        state.set_state_value(any_foo.write_to_bytes().unwrap());

        state
    }

    /// It's important to create multiple invocations to test whether state updates can be "seen"
    /// by later invocations in a batch.
    fn invocations() -> RepeatedField<ToFunction_Invocation> {
        let mut invocations = RepeatedField::new();

        invocations.push(invocation(caller_address(), MESSAGE1));
        invocations.push(invocation(caller_address(), MESSAGE2));
        invocations.push(invocation(caller_address(), MESSAGE3));

        invocations
    }

    fn invocation(caller: Address, argument: &str) -> ToFunction_Invocation {
        let mut invocation = ToFunction_Invocation::new();

        let message = string_value(argument);
        let packed_argument = Any::pack(&message).unwrap();
        invocation.set_caller(caller.into_proto());
        invocation.set_argument(packed_argument);

        invocation
    }

    fn string_value(value: &str) -> StringValue {
        // StringValue (and generated code from protobuf in general) is not very ergonomic...
        let mut result = StringValue::new();
        result.set_value(value.to_owned());
        result
    }

    fn i32_value(value: i32) -> Int32Value {
        let mut result = Int32Value::new();
        result.set_value(value);
        result
    }

    fn unpack_any<M: Message>(any: &Any) -> M {
        any.unpack()
            .expect("Could not unwrap Result")
            .expect("Could not unwrap Option.")
    }
}
