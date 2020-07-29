use protobuf::ProtobufError;
use thiserror::Error;

use crate::FunctionType;

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
}
