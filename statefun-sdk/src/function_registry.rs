use std::collections::HashMap;

use failure::format_err;
use protobuf::Message;

use statefun_protos::http_function::FromFunction;
use statefun_protos::http_function::FromFunction_EgressMessage;
use statefun_protos::http_function::FromFunction_InvocationResponse;
use statefun_protos::http_function::FromFunction_PersistedValueMutation;
use statefun_protos::http_function::FromFunction_PersistedValueMutation_MutationType;
use statefun_protos::http_function::ToFunction;
use statefun_protos::http_function::ToFunction_PersistedValue;

use crate::{Context, Effects, EgressIdentifier, FunctionType, StateUpdate};
use protobuf::well_known_types::Any;

#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<FunctionType, Box<dyn InvokableFunction + Send>>,
}

impl FunctionRegistry {
    pub fn new() -> FunctionRegistry {
        FunctionRegistry {
            functions: HashMap::new(),
        }
    }

    pub fn register_fn<I: Message, F: Fn(Context, I) -> Effects + Send + 'static>(
        &mut self,
        function_type: FunctionType,
        function: F,
    ) {
        let callable_function = FnInvokableFunction {
            function,
            marker: ::std::marker::PhantomData,
        };
        self.functions
            .insert(function_type, Box::new(callable_function));
    }

    pub fn invoke(&self, to_function: ToFunction) -> Result<FromFunction, failure::Error> {
        let batch_request = to_function.get_invocation();
        log::debug!(
            "FunctionRegistry: processing batch request {:#?}",
            batch_request
        );

        let target = batch_request.get_target();
        let namespace = target.get_namespace();
        let name = target.get_field_type();

        let function_type = FunctionType::new(namespace, name);
        let function = self.functions.get(&function_type);
        match function {
            Some(fun) => fun.invoke(to_function),
            None => Err(format_err!(
                "No function registered under {}",
                function_type
            )),
        }
    }
}

trait InvokableFunction {
    fn invoke(&self, to_function: ToFunction) -> Result<FromFunction, failure::Error>;
}

struct FnInvokableFunction<I: Message, F: Fn(Context, I) -> Effects> {
    function: F,
    marker: ::std::marker::PhantomData<I>,
}

impl<I: Message, F: Fn(Context, I) -> Effects> InvokableFunction for FnInvokableFunction<I, F> {
    fn invoke(&self, to_function: ToFunction) -> Result<FromFunction, failure::Error> {
        let batch_request = to_function.get_invocation();
        log::debug!(
            "CallableFunction: processing batch request {:#?}",
            batch_request
        );

        let self_address = batch_request.get_target();
        let persisted_values = parse_persisted_values(batch_request.get_state());

        let mut invocation_respose = FromFunction_InvocationResponse::new();

        for invocation in batch_request.get_invocations() {
            let caller_address = invocation.get_caller();
            let argument = invocation.get_argument();
            let unpacked_argument: I = argument.unpack()?.unwrap();
            let context = Context {
                state: &persisted_values,
                self_address,
                caller_address,
            };
            let effects = (self.function)(context, unpacked_argument);

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
