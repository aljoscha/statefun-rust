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
    fn invoke_old(&self, to_function: ToFunction) -> Result<FromFunction, failure::Error>;
}

impl InvocationBridge for FunctionRegistry {
    fn invoke_old(&self, mut to_function: ToFunction) -> Result<FromFunction, failure::Error> {
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
