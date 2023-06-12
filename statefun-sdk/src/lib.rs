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
use statefun_proto::types::{BooleanWrapper, IntWrapper, LongWrapper};

mod serialization;
use serialization::{serializer, deserializer};
pub use serialization::Serializable;
mod function_type;
pub use function_type::FunctionType;
mod address;
pub use address::Address;
mod egress_identifier;
pub use egress_identifier::EgressIdentifier;
mod context;
pub use context::Context;
mod error;
mod effects;
pub use effects::Effects;
mod function_registry;
mod invocation_bridge;
pub mod io;
pub mod transport;
use statefun_proto::request_reply::TypedValue;

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

///
#[derive(Debug)]
pub struct StateMessage {
    typed_value : TypedValue,
}

impl StateMessage {
    ///
    pub fn get<T : Serializable>(&self) -> Option<T> {
        // todo: make deserializer return Option
        Some(deserializer::<T>(self.typed_value.typename.to_string(), &self.typed_value.value))
    }

    ///
    pub fn new(typed_value: TypedValue) -> Self {
        StateMessage {
            typed_value: typed_value
        }
    }
}

#[derive(Debug)]
enum StateUpdate {
    Update(ValueSpecBase, Vec<u8>),
    Delete(ValueSpecBase),
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

impl<T: Serializable> ValueSpec<T> {
    // todo: could make this a trait by implementing as_const_str() on a static str
    ///
    pub const fn new(name: &'static str, built_in_type: BuiltInTypes) -> ValueSpec<T> {
        ValueSpec {
            name: name,
            typename: built_in_type.as_const_str(),
            serializer: serializer,
            deserializer: deserializer,
        }
    }

    ///
    pub const fn custom(name: &'static str, typename: &'static str) -> ValueSpec<T> {
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
