use protobuf::ProtobufError;
use thiserror::Error;
use crate::FunctionType;
use crate::MissingStateCollection;
use std::fmt::{Display, Formatter};

/// Errors that can occur during function invocation.
///
/// These mostly forward underlying errors from serialization or Protobuf.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum InvocationError {
    /// There was no function registered for the given `FunctionType`.
    #[error("function {0} not found in registry")]
    FunctionNotFound(FunctionType),

    /// Something went wrong with Protobuf parsing, writing, packing, or unpacking.
    #[error(transparent)]
    ProtobufError(#[from] ProtobufError),

    /// Missing state, ask Flink to prepare state storage and it will initiate the call again
    #[error(transparent)]
    MissingStates(MissingStateCollection)
}

/// Errors that can occur during serialization / deserialization.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SerializationError {
    ///
    #[error(transparent)]
    SerializationError(SomeError),

    ///
    #[error(transparent)]
    DeserializationError(SomeError),
}

///
#[derive(Error, PartialEq, Eq, Hash, Debug)]
pub struct SomeError {

}

impl SomeError {
    ///
    pub fn new() -> SomeError {
        SomeError {}
    }
}

impl Display for SomeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SomeError")
    }
}
