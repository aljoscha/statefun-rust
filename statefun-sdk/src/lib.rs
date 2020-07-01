use std::fmt::{Display, Formatter};

use protobuf::well_known_types::Any;
use protobuf::Message;

pub use function_registry::FunctionRegistry;
use std::collections::HashMap;

mod function_registry;
pub mod io;
pub mod transport;

pub struct Context<'a> {
    state: &'a HashMap<String, Vec<u8>>,
}

impl<'a> Context<'a> {
    pub fn self_address() -> Address {
        unimplemented!()
    }

    pub fn caller_address() -> Address {
        unimplemented!()
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

pub struct Address {
    pub function_type: FunctionType,
    pub id: String,
}

#[derive(Default)]
pub struct Effects {
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
            egress_messages: Vec::new(),
            state_updates: Vec::new(),
        }
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
