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
use protobuf::parse_from_bytes;
use thiserror::Error;

pub use error::InvocationError;
pub use function_registry::FunctionRegistry;
use statefun_proto::request_reply::Address as ProtoAddress;
use statefun_proto::types::{BooleanWrapper, IntWrapper};

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
    state: &'a HashMap<ValueSpecBase, Vec<u8>>,
    self_address: &'a ProtoAddress,
    caller_address: &'a ProtoAddress,
}

impl<'a> Context<'a> {
    fn new(
        state: &'a HashMap<ValueSpecBase, Vec<u8>>,
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
    pub fn get_state<T>(&self, value_spec: ValueSpec<T>) -> Option<T> {
        let deserializer = value_spec.deserializer.clone();
        let typename = value_spec.typename.to_string();
        let state = self.state.get(&value_spec.into());
        match state {
            Some(serialized) => {
                let deserialized : T = deserializer(typename, serialized);
                Some(deserialized)
            }
            None => None
        }

        // let deserialized = (value_spec.deserializer)(state);
        // self.state_updates.push(StateUpdate::Update(
        //     value_spec.into(),
        //     serialized,
        // ));
        // None

        // todo: deserialize with user-provided serializer
        // state.and_then(|serialized_state| {
        //     let unpacked_state: Option<T> = unpack_state(value_spec, serialized_state);
        //     unpacked_state
        // })
    }
}

/// Unpacks the given state, which is expected to be a serialized `Any<T>`.
fn unpack_state<T: Message>(value_spec: ValueSpecBase, packed_state: &Any) -> Option<T> {
    // let packed_state: Any =
    //     protobuf::parse_from_bytes(serialized_state).expect("Could not deserialize state.");

    log::debug!("Packed state for {:?}: {:?}", value_spec, packed_state);

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

/// blabla
#[derive(Error, PartialEq, Eq, Hash, Debug)]
pub struct MissingStateCollection {
    states: Vec<ValueSpecBase>,
}

impl MissingStateCollection {
    /// blabla
    pub fn new(states: Vec<ValueSpecBase>) -> MissingStateCollection {
        MissingStateCollection {
            states: states,
        }
    }
}

impl Display for MissingStateCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MissingStateCollection {:?}", self.states)
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
    invocations: Vec<(Address, String, Any)>,
    delayed_invocations: Vec<(Address, Duration, String, Any)>,
    egress_messages: Vec<(EgressIdentifier, String, Any)>,
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
    // todo: check if this needs to be valuespec in the java sdk
    pub fn send<M: Message>(&mut self, address: Address, value_spec: ValueSpecBase, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.invocations.push((address, value_spec.typename, packed_message));
    }

    /// Sends a message to the stateful function identified by the address after a delay.
    pub fn send_after<M: Message>(&mut self, address: Address, delay: Duration, value_spec: ValueSpecBase, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.delayed_invocations
            .push((address, delay, value_spec.typename, packed_message));
    }

    /// Sends a message to the egress identifier by the `EgressIdentifier`.
    pub fn egress<M: Message>(&mut self, identifier: EgressIdentifier, value_spec: ValueSpecBase, message: M) {
        let packed_message = Any::pack(&message).unwrap();
        self.egress_messages.push((identifier, value_spec.typename, packed_message));
    }

    /// Deletes the state kept under the given name.
    pub fn delete_state(&mut self, value_spec: ValueSpecBase) {
        self.state_updates
            .push(StateUpdate::Delete(value_spec));
    }

    /// Updates the state stored under the given name to the given value.
    pub fn update_state<T>(&mut self, value_spec: ValueSpec<T>, value: &T) {
        let serialized = (value_spec.serializer)(value, value_spec.typename.to_string());
        log::debug!("-- drey: updated state: {:?}", serialized);
        self.state_updates.push(StateUpdate::Update(
            value_spec.into(),
            serialized,
        ));
    }
}

#[derive(Debug)]
enum StateUpdate {
    Update(ValueSpecBase, Vec<u8>),
    Delete(ValueSpecBase),
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

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpecBase {
    name : String,  // state name
    typename : String,  // type typename
}

impl ValueSpecBase {
    ///
    fn new(name: &str, typename: &str) -> ValueSpecBase {
        ValueSpecBase {
            name: name.to_string(),
            typename: typename.to_string(),
        }
    }
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpec<T> {
    name : &'static str,  // state name
    typename : &'static str,  // type typename

    // todo: should these implement Result?
    serializer: fn(&T, String) -> Vec<u8>,
    deserializer: fn(String, &Vec<u8>) -> T,
}

///
impl<T> Into<ValueSpecBase> for ValueSpec<T> {
    ///
    fn into(self) -> ValueSpecBase {
        ValueSpecBase::new(self.name.to_string().as_str(), self.typename.to_string().as_str())
    }
}

trait Serializable {
    fn serialize(&self, typename: String) -> Vec<u8>;
    fn deserialize(typename: String, buffer: &Vec<u8>) -> Self;
}

impl Serializable for bool {
    fn serialize(&self, typename: String) -> Vec<u8> {
        let mut wrapped = BooleanWrapper::new();
        wrapped.set_value(*self);
        wrapped.write_to_bytes().unwrap()
    }

    fn deserialize(typename: String, buffer: &Vec<u8>) -> bool {
        let wrapped = parse_from_bytes::<BooleanWrapper>(&buffer).unwrap();
        wrapped.get_value()
    }
}

impl Serializable for i32 {
    fn serialize(&self, typename: String) -> Vec<u8> {
        let mut wrapped = IntWrapper::new();
        log::debug!("-- drey: i32 serializing {:?}", self);
        wrapped.set_value(*self);
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        res
    }

    fn deserialize(typename: String, buffer: &Vec<u8>) -> i32 {
        let wrapped = parse_from_bytes::<IntWrapper>(&buffer).unwrap();
        wrapped.get_value()
    }
}

fn builtin_serializer<T : Serializable>(value: &T, typename: String) -> Vec<u8> {
    // log::debug!("-- drey: serializing type: {:?}", typename);
    (&value).serialize(typename)
    // log::debug!("-- drey: serialized to: {:?}", &res);
}

// todo
fn builtin_deserializer<T : Serializable>(typename: String, buffer: &Vec<u8>) -> T {
    // log::debug!("-- drey: deserializing type: {:?}", typename);
    // todo: how do we limit T here so T::new will work??
    // T::new()
    // panic!("oops")

    T::deserialize(typename, buffer)
}

impl<T: Serializable> ValueSpec<T> {
    /// todo: there's no function overloading in Rust, what to do here to make this nicer?
    pub const fn new(name: &'static str, built_in_type: BuiltInTypes) -> ValueSpec<T> {
        ValueSpec {
            name: name,
            typename: built_in_type.as_const_str(),
            serializer: builtin_serializer,
            deserializer: builtin_deserializer,
        }
    }

    ///
    fn custom(name: &'static str, typename: &'static str, serializer: fn(&T, String) -> Vec<u8>, deserializer: fn(String, &Vec<u8>) -> T) -> ValueSpec<T> {
        ValueSpec {
            name: name,
            typename: typename,
            serializer: serializer,
            deserializer: deserializer,
        }
    }

    fn as_base(&self) -> ValueSpecBase {
        ValueSpecBase {
            name : self.name.to_string(),
            typename : self.typename.to_string(),
        }
    }
}

///
pub enum BuiltInTypes {
    ///
    Boolean,
    ///
    Integer,
    ///
    Long,
    ///
    Float,
    ///
    Double,
    ///
    String,
}

impl BuiltInTypes {
    const fn as_const_str(&self) -> &'static str {
        match self {
            BuiltInTypes::Boolean => "io.statefun.types/bool",
            BuiltInTypes::Integer => "io.statefun.types/int",
            BuiltInTypes::Long => "io.statefun.types/long",
            BuiltInTypes::Float => "io.statefun.types/float",
            BuiltInTypes::Double => "io.statefun.types/double",
            BuiltInTypes::String => "io.statefun.types/string",
        }
    }
}

fn from_str(input: String) -> BuiltInTypes {
    match input.as_str() {
        "io.statefun.types/bool" => BuiltInTypes::Boolean,
        "io.statefun.types/int" => BuiltInTypes::Integer,
        "io.statefun.types/long" => BuiltInTypes::Long,
        "io.statefun.types/float" => BuiltInTypes::Float,
        "io.statefun.types/double" => BuiltInTypes::Double,
        "io.statefun.types/string" => BuiltInTypes::String,
        _ => panic!("Unexpected type")
    }
}
