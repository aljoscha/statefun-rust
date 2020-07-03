//! Transports are used to serve stateful functions to make them invokable.

use crate::internal::FunctionRegistry;

pub mod hyper;

/// Serves up stateful functions in a [FunctionRegistry](crate::FunctionRegistry) to make them
/// invokable in a Statefun deployment.
pub trait Transport {
    /// Serves the stateful functions in the given `FunctionRegistry`. This will usually be a
    /// blocking method and should be the last method you call in your program.
    fn run(self, function_registry: FunctionRegistry) -> Result<(), failure::Error>;
}
