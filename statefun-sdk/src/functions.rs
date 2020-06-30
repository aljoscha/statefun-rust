use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use failure::format_err;
use protobuf::well_known_types::Any;
use protobuf::Message;

use statefun_protos::http_function::FromFunction;
use statefun_protos::http_function::FromFunction_EgressMessage;
use statefun_protos::http_function::FromFunction_InvocationResponse;
use statefun_protos::http_function::ToFunction;

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

pub struct Context {}

impl Context {
    pub fn self_address() -> Address {
        unimplemented!()
    }

    pub fn caller_address() -> Address {
        unimplemented!()
    }
}

pub struct Address {
    pub function_type: FunctionType,
    pub id: String,
}

#[derive(Default)]
pub struct Effects {
    egress_messages: Vec<(EgressIdentifier, Any)>,
}

impl Effects {
    pub fn new() -> Effects {
        Effects {
            egress_messages: Vec::new(),
        }
    }

    pub fn egress<M: Message>(&mut self, identifier: EgressIdentifier, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.egress_messages.push((identifier, packed_message));
    }
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

        let mut invocation_respose = FromFunction_InvocationResponse::new();

        for invocation in batch_request.get_invocations() {
            let argument = invocation.get_argument();
            let unpacked_argument: I = argument.unpack()?.unwrap();
            let context = Context {};
            let effects = (self.function)(context, unpacked_argument);

            for egress_message in effects.egress_messages {
                let mut proto_egress_message = FromFunction_EgressMessage::new();
                proto_egress_message.set_egress_namespace(egress_message.0.namespace);
                proto_egress_message.set_egress_type(egress_message.0.name);
                proto_egress_message.set_argument(egress_message.1);
                invocation_respose
                    .outgoing_egresses
                    .push(proto_egress_message);
            }
        }

        let mut from_function = FromFunction::new();
        from_function.set_invocation_result(invocation_respose);

        Ok(from_function)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct FunctionType {
    namespace: String,
    name: String,
}

impl FunctionType {
    pub fn new(namespace: &str, name: &str) -> FunctionType {
        FunctionType {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionType {}/{}", self.namespace, self.name)
    }
}

pub struct EgressIdentifier {
    namespace: String,
    name: String,
}

impl EgressIdentifier {
    pub fn new(namespace: &str, name: &str) -> EgressIdentifier {
        EgressIdentifier {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }
}

impl Display for EgressIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EgressIdentifier {}/{}", self.namespace, self.name)
    }
}
