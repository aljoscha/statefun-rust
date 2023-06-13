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

pub use error::InvocationError;
pub use function_registry::FunctionRegistry;

mod built_in_types;
mod serialization;
pub use built_in_types::BuiltInTypes;
pub use serialization::Serializable;
use serialization::{deserializer, serializer};
mod state_message;
pub use state_message::StateMessage;
mod function_type;
pub use function_type::FunctionType;
mod address;
mod type_name;
pub use type_name::TypeName;
pub use address::Address;
mod egress_identifier;
pub use egress_identifier::EgressIdentifier;
mod context;
mod state_update;
mod value_spec;
use state_update::StateUpdate;
mod value_spec_base;
pub use context::Context;
pub use value_spec::ValueSpec;
pub use value_spec_base::ValueSpecBase;
mod effects;
mod error;
pub use effects::Effects;
mod function_registry;
mod invocation_bridge;
mod missing_state_collection;
use missing_state_collection::MissingStateCollection;
pub mod io;
pub mod transport;
use statefun_proto::request_reply::TypedValue;
