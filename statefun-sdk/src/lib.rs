use std::fmt::{Display, Formatter};

use protobuf::well_known_types::Any;
use protobuf::Message;

pub use function_registry::FunctionRegistry;

mod function_registry;
pub mod io;
pub mod transport;

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
