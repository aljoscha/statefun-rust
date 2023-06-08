//! An SDK for writing "stateful functions" in Rust. For use with
//! [Apache Flink Stateful Functions](https://flink.apache.org/stateful-functions.html) (Statefun).
//!
//! # Examples
//!
//! The following shows how to write a simple stateful function and serve it for use in a Statefun
//! deployment.
//!
//! ```no_run
//! use protobuf::well_known_types::StringValue;
//!
//! use statefun::io::kafka;
//! use statefun::transport::hyper::HyperHttpTransport;
//! use statefun::transport::Transport;
//! use statefun::{Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType};
//!
//! let mut function_registry = FunctionRegistry::new();
//!
//! function_registry.register_fn(
//!     FunctionType::new("example", "function1"),
//!     |context, message: StringValue| {
//!         let mut effects = Effects::new();
//!
//!         effects.send(
//!             Address::new(FunctionType::new("example", "function2"), "doctor"),
//!             message,
//!         );
//!
//!         effects
//!     },
//! );
//!
//! let hyper_transport = HyperHttpTransport::new("0.0.0.0:5000".parse()?);
//! hyper_transport.run(function_registry)?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! The program creates a [FunctionRegistry](crate::FunctionRegistry), which can be used to
//! register one or more functions. Then we register a closure as a stateful function. Finally,
//! we need to create a [Transport](crate::transport::Transport), in this case the
//! [HyperHttpTransport](crate::transport::hyper::HyperHttpTransport) to serve our stateful
//! function.
//!
//! Not that you can also use a function instead of a closure when registering functions.
//!
//! Refer to the Stateful Functions
//! [documentation](https://ci.apache.org/projects/flink/flink-statefun-docs-master/) to learn how
//! to use this in a deployment. Especially the
//! [modules documentation](https://ci.apache.org/projects/flink/flink-statefun-docs-master/sdk/modules.html#remote-module) is pertinent.

#![deny(missing_docs)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use protobuf::well_known_types::Any;
use protobuf::Message;

pub use error::InvocationError;
pub use function_registry::FunctionRegistry;
use statefun_proto::request_reply::Address as ProtoAddress;

mod error;
mod function_registry;
mod invocation_bridge;

pub mod io;
pub mod transport;

/// Context for a single invocation of a stateful function.
///
/// The context may be used to obtain the [Address](Address) of the function of the current
/// invocation or the calling function (if the function was invoked by another function), or to
/// access state.
#[derive(Debug)]
pub struct Context<'a> {
    state: &'a HashMap<String, Any>,
    self_address: &'a ProtoAddress,
    caller_address: &'a ProtoAddress,
}

impl<'a> Context<'a> {
    fn new(
        state: &'a HashMap<String, Any>,
        self_address: &'a ProtoAddress,
        caller_address: &'a ProtoAddress,
    ) -> Self {
        Context {
            state,
            self_address,
            caller_address,
        }
    }

    /// Returns the [Address](Address) of the stateful function that is being called. This is the
    /// statefun equivalent of `self`.
    pub fn self_address(&self) -> Address {
        Address::from_proto(self.self_address)
    }

    /// Returns the [Address](Address) of the stateful function that caused this function
    /// invocation, that is, the caller.
    pub fn caller_address(&self) -> Address {
        Address::from_proto(self.caller_address)
    }

    /// Returns the state (or persisted) value that previous invocations of this stateful function
    /// might have persisted under the given name.
    pub fn get_state<T: Message>(&self, name: &str) -> Option<T> {
        let state = self.state.get(name);
        state.and_then(|serialized_state| {
            let unpacked_state: Option<T> = unpack_state(name, serialized_state);
            unpacked_state
        })
    }
}

/// Unpacks the given state, which is expected to be a serialized `Any<T>`.
fn unpack_state<T: Message>(state_name: &str, packed_state: &Any) -> Option<T> {
    // let packed_state: Any =
    //     protobuf::parse_from_bytes(serialized_state).expect("Could not deserialize state.");

    log::debug!("Packed state for {}: {:?}", state_name, packed_state);

    let unpacked_state: Option<T> = packed_state
        .unpack()
        .expect("Could not unpack state from Any.");

    unpacked_state
}

/// The unique identity of an individual stateful function.
///
/// This comprises the function's `FunctionType` and an unique identifier within the
/// type. The function's type denotes the class of function to invoke, while the unique identifier
/// addresses the invocation to a specific function instance.
///
/// This must be used when sending messages to stateful functions as part of the function
/// [Effects](Effects).
#[derive(Debug, PartialEq)]
pub struct Address {
    /// `FunctionType` of the stateful function that this `Address` refers to.
    pub function_type: FunctionType,

    /// Unique id of the stateful function that this `Address` refers to.
    pub id: String,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address {}/{}", self.function_type, self.id)
    }
}

impl Address {
    /// Creates a new `Address` from the given `FunctionType` and id.
    pub fn new(function_type: FunctionType, id: &str) -> Self {
        Address {
            function_type,
            id: id.to_owned(),
        }
    }

    /// Converts the Protobuf `Address` into an `Address`. We don't implement `From`/`Into` for this
    /// because we want to keep it out of the public API.
    fn from_proto(proto_address: &ProtoAddress) -> Self {
        Address {
            function_type: FunctionType::new(
                proto_address.get_namespace(),
                proto_address.get_field_type(),
            ),
            id: proto_address.get_id().to_owned(),
        }
    }

    /// Converts this `Address` into a Protobuf `Address`. We don't implement `From`/`Into` for this
    /// because we want to keep it out of the public API.
    fn into_proto(self) -> ProtoAddress {
        let mut result = ProtoAddress::new();
        result.set_namespace(self.function_type.namespace);
        result.set_field_type(self.function_type.name);
        result.set_id(self.id);
        result
    }
}

/// A reference to a stateful function, consisting of a namespace and a name.
///
/// A function's type is part of a function's [Address](Address) and serves as integral part of an
/// individual function's identity.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct FunctionType {
    namespace: String,
    name: String,
}

impl FunctionType {
    /// Creates a new `FunctionType` from the given namespace and name.
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

/// Effects (or side effects) of a stateful function invocation.
///
/// This can be used to:
///  - send messages ourselves or other stateful functions
///  - send messages to an egress
///  - update the state of this stateful function, which will be available on future invocations
#[derive(Default, Debug)]
pub struct Effects {
    invocations: Vec<(Address, Any)>,
    delayed_invocations: Vec<(Address, Duration, Any)>,
    egress_messages: Vec<(EgressIdentifier, Any)>,
    state_updates: Vec<StateUpdate>,
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
    pub fn send<M: Message>(&mut self, address: Address, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.invocations.push((address, packed_message));
    }

    /// Sends a message to the stateful function identified by the address after a delay.
    pub fn send_after<M: Message>(&mut self, address: Address, delay: Duration, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.delayed_invocations
            .push((address, delay, packed_message));
    }

    /// Sends a message to the egress identifier by the `EgressIdentifier`.
    pub fn egress<M: Message>(&mut self, identifier: EgressIdentifier, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.egress_messages.push((identifier, packed_message));
    }

    /// Deletes the state kept under the given name.
    pub fn delete_state(&mut self, name: &str) {
        self.state_updates
            .push(StateUpdate::Delete(name.to_owned()));
    }

    /// Updates the state stored under the given name to the given value.
    pub fn update_state<T: Message>(&mut self, name: &str, value: &T) {
        self.state_updates.push(StateUpdate::Update(
            name.to_owned(),
            Any::pack(value).expect("Could not pack state update."),
        ));
    }
}

#[derive(Debug)]
enum StateUpdate {
    Update(String, Any),
    Delete(String),
}

/// A reference to an _egress_, consisting of a namespace and a name.
///
/// This has to be used when sending messages to an egress as part of the function
/// [Effects](Effects).
#[derive(Debug)]
pub struct EgressIdentifier {
    namespace: String,
    name: String,
}

impl EgressIdentifier {
    /// Creates a new `EgressIdentifier` from the given namespace and name.
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
