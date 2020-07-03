//! The function registry keeps a mapping from `FunctionType` to stateful functions.

use std::collections::HashMap;

use failure::format_err;
use protobuf::well_known_types::Any;
use protobuf::Message;

use crate::{Context, Effects, FunctionType};

/// Keeps a mapping from `FunctionType` to stateful functions. Use this together with a
/// [Transport](crate::transport::Transport) to serve stateful functions.
///
/// Use `register_fn()` to register functions before handing the registry over to a `Transport` for
/// serving.
#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<FunctionType, Box<dyn InvokableFunction + Send>>,
}

impl FunctionRegistry {
    /// Creates a new empty `FunctionRegistry`.
    pub fn new() -> FunctionRegistry {
        FunctionRegistry {
            functions: HashMap::new(),
        }
    }

    /// Registers the given function under the `function_type`.
    pub fn register_fn<I: Message, F: Fn(Context, I) -> Effects + Send + 'static>(
        &mut self,
        function_type: FunctionType,
        function: F,
    ) {
        let callable_function = FnInvokableFunction {
            function,
            marker: ::std::marker::PhantomData,
        };
        self.functions
            .insert(function_type, Box::new(callable_function));
    }

    /// Invokes the function that is registered for the given `FunctionType`. This will return
    /// `Err` if no function is registered under the given type.
    pub fn invoke(
        &self,
        target_function: FunctionType,
        context: Context,
        message: Any,
    ) -> Result<Effects, failure::Error> {
        let function = self.functions.get(&target_function);
        match function {
            Some(fun) => fun.invoke(context, message),
            None => Err(format_err!(
                "No function registered under {}",
                target_function
            )),
        }
    }
}

/// A function that can be invoked. This is used as trait objects in the `FunctionRegistry`.
trait InvokableFunction {
    fn invoke(&self, context: Context, message: Any) -> Result<Effects, failure::Error>;
}

/// An `InvokableFunction` that is backed by a `Fn`.
struct FnInvokableFunction<I: Message, F: Fn(Context, I) -> Effects> {
    function: F,
    marker: ::std::marker::PhantomData<I>,
}

impl<I: Message, F: Fn(Context, I) -> Effects> InvokableFunction for FnInvokableFunction<I, F> {
    fn invoke(&self, context: Context, message: Any) -> Result<Effects, failure::Error> {
        let unpacked_argument: I = message.unpack()?.unwrap();
        let effects = (self.function)(context, unpacked_argument);
        Ok(effects)
    }
}
