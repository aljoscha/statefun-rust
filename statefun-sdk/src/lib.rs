//! An SDK for writing "stateful functions" in Rust. For use with
//! [Apache Flink Stateful Functions](https://flink.apache.org/stateful-functions.html) (Statefun).
//!
//! # Examples
//!
//! Please see the `statefun-greeter-example` for how to use this library.
//!
//! The program creates a [FunctionRegistry](crate::FunctionRegistry), which can be used to
//! register one or more functions. Then we register a closure as a stateful function. Finally,
//! we need to create a [Transport](crate::transport::Transport), in this case the
//! [HyperHttpTransport](crate::transport::hyper::HyperHttpTransport) to serve our stateful
//! function.
//!
//! Note that you can also use a function instead of a closure when registering functions.
//!
//! Refer to the Stateful Functions
//! [documentation](https://ci.apache.org/projects/flink/flink-statefun-docs-master/) to learn how
//! to use this in a deployment. Especially the
//! [modules documentation](https://ci.apache.org/projects/flink/flink-statefun-docs-master/sdk/modules.html#remote-module) is pertinent.

#![deny(missing_docs)]

pub mod io;
pub mod transport;

pub use crate::transport::hyper::HyperHttpTransport;
pub use address::Address;
pub use context::Context;
pub use effects::Effects;
pub use egress_identifier::EgressIdentifier;
pub use function_registry::FunctionRegistry;
pub use function_type::FunctionType;
pub use message::Message;
pub use traits::{GetTypename, Serializable};
pub use type_spec::TypeSpec;
pub use value_spec::ValueSpec;
pub use expiration::{Expiration, ExpirationType};

mod address;
mod context;
mod effects;
mod expiration;
mod egress_identifier;
mod error;
mod function_registry;
mod function_type;
mod invocation_bridge;
mod message;
mod missing_states;
mod serialization;
mod state_update;
mod traits;
mod delayed_invocation;
mod type_spec;
mod value_spec;
mod value_spec_base;

use error::InvocationError;
use missing_states::MissingStates;
use state_update::StateUpdate;
use statefun_proto::request_reply::TypedValue;
use value_spec_base::ValueSpecBase;
use delayed_invocation::DelayedInvocation;
