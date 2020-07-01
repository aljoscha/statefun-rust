use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use protobuf::well_known_types::Any;
use protobuf::Message;

pub use internal::FunctionRegistry;
use statefun_protos::http_function::Address as ProtoAddress;

mod internal;
pub mod io;
pub mod transport;

pub struct Context<'a> {
    state: &'a HashMap<&'a str, &'a [u8]>,
    self_address: &'a ProtoAddress,
    caller_address: &'a ProtoAddress,
}

impl<'a> Context<'a> {
    pub fn self_address(&self) -> Address {
        Address::from_proto(self.self_address)
    }

    pub fn caller_address(&self) -> Address {
        Address::from_proto(self.caller_address)
    }

    pub fn get_state<T: Message>(&self, name: &str) -> Option<T> {
        let state = self.state.get(name);
        state.and_then(|serialized_state| {
            let packed_state: Any =
                protobuf::parse_from_bytes(serialized_state).expect("Could not deserialize state.");

            log::debug!("Packed state for {}: {:?}", name, packed_state);

            let unpacked_state: Option<T> = packed_state
                .unpack()
                .expect("Could not unpack state from Any.");

            unpacked_state
        })
    }
}

#[derive(Debug)]
pub struct Address {
    pub function_type: FunctionType,
    pub id: String,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address {}/{}", self.function_type, self.id)
    }
}

impl Address {
    fn from_proto(proto_address: &ProtoAddress) -> Self {
        Address {
            function_type: FunctionType::new(
                proto_address.get_namespace(),
                proto_address.get_field_type(),
            ),
            id: proto_address.get_id().to_owned(),
        }
    }

    fn into_proto(self) -> ProtoAddress {
        let mut result = ProtoAddress::new();
        result.set_namespace(self.function_type.namespace);
        result.set_field_type(self.function_type.name);
        result.set_id(self.id);
        result
    }
}

#[derive(Default)]
pub struct Effects {
    invocations: Vec<(Address, Any)>,
    egress_messages: Vec<(EgressIdentifier, Any)>,
    state_updates: Vec<StateUpdate>,
}

enum StateUpdate {
    Update(String, Any),
    Delete(String),
}

impl Effects {
    pub fn new() -> Effects {
        Effects {
            invocations: Vec::new(),
            egress_messages: Vec::new(),
            state_updates: Vec::new(),
        }
    }

    pub fn send<M: Message>(&mut self, address: Address, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.invocations.push((address, packed_message));
    }

    pub fn egress<M: Message>(&mut self, identifier: EgressIdentifier, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.egress_messages.push((identifier, packed_message));
    }

    pub fn delete_state(&mut self, name: &str) {
        self.state_updates
            .push(StateUpdate::Delete(name.to_owned()));
    }

    pub fn update_state<T: Message>(&mut self, name: &str, value: &T) {
        self.state_updates.push(StateUpdate::Update(
            name.to_owned(),
            Any::pack(value).expect("Could not pack state update."),
        ));
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
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
