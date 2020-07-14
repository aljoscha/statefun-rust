//! The function registry keeps a mapping from `FunctionType` to stateful functions.

use std::collections::HashMap;

use protobuf::well_known_types::Any;
use protobuf::Message;

use crate::InvocationError::FunctionNotFound;
use crate::{Context, Effects, FunctionType, InvocationError};

/// Keeps a mapping from `FunctionType` to stateful functions. Use this together with a
/// [Transport](crate::transport::Transport) to serve stateful functions.
///
/// Use `register_fn()` to register functions before handing the registry over to a `Transport` for
/// serving.
pub struct FunctionRegistry {
    functions: HashMap<FunctionType, Box<dyn InvokableFunction + Send>>,
}

#[allow(clippy::new_without_default)]
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
    ) -> Result<Effects, InvocationError> {
        let function = self.functions.get(&target_function);
        match function {
            Some(fun) => fun.invoke(context, message),
            None => Err(FunctionNotFound(target_function)),
        }
    }
}

/// A function that can be invoked. This is used as trait objects in the `FunctionRegistry`.
trait InvokableFunction {
    fn invoke(&self, context: Context, message: Any) -> Result<Effects, InvocationError>;
}

/// An `InvokableFunction` that is backed by a `Fn`.
struct FnInvokableFunction<I: Message, F: Fn(Context, I) -> Effects> {
    function: F,
    marker: ::std::marker::PhantomData<I>,
}

impl<I: Message, F: Fn(Context, I) -> Effects> InvokableFunction for FnInvokableFunction<I, F> {
    fn invoke(&self, context: Context, message: Any) -> Result<Effects, InvocationError> {
        let unpacked_argument: I = message.unpack()?.unwrap();
        let effects = (self.function)(context, unpacked_argument);
        Ok(effects)
    }
}

#[cfg(test)]
mod tests {
    use protobuf::well_known_types::StringValue;

    use crate::FunctionRegistry;
    use crate::*;

    #[test]
    fn call_registered_function() -> anyhow::Result<()> {
        let state = HashMap::new();
        let address = address_foo().into_proto();
        let context = Context::new(&state, &address, &address);

        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type_foo(), |_context, _message: StringValue| {
            Effects::new()
        });

        let packed_argument = Any::pack(&StringValue::new())?;
        let _effects = registry.invoke(function_type_foo(), context, packed_argument)?;

        Ok(())
    }

    #[test]
    fn call_unknown_function() -> anyhow::Result<()> {
        let state = HashMap::new();
        let address = address_foo().into_proto();
        let context = Context::new(&state, &address, &address);

        let registry = FunctionRegistry::new();

        let packed_argument = Any::pack(&StringValue::new())?;
        let result = registry.invoke(function_type_bar(), context, packed_argument);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn call_correct_function() -> anyhow::Result<()> {
        let state = HashMap::new();

        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type_foo(), |context, _message: StringValue| {
            let mut effects = Effects::new();

            let mut message = StringValue::new();
            message.set_value("function_foo".to_owned());
            effects.send(context.self_address(), message);

            effects
        });

        registry.register_fn(function_type_bar(), |context, _message: StringValue| {
            let mut effects = Effects::new();

            let mut message = StringValue::new();
            message.set_value("function_bar".to_owned());
            effects.send(context.self_address(), message);

            effects
        });

        let address_foo = address_foo().into_proto();
        let context = Context::new(&state, &address_foo, &address_foo);
        let packed_argument = Any::pack(&StringValue::new())?;
        let effects_foo = registry.invoke(function_type_foo(), context, packed_argument)?;
        assert_eq!(
            effects_foo.invocations[0]
                .1
                .unpack::<StringValue>()
                .unwrap()
                .unwrap()
                .get_value(),
            "function_foo",
        );

        let address_bar = address_bar().into_proto();
        let context = Context::new(&state, &address_bar, &address_bar);
        let packed_argument = Any::pack(&StringValue::new())?;
        let effects_bar = registry.invoke(function_type_bar(), context, packed_argument)?;
        assert_eq!(
            effects_bar.invocations[0]
                .1
                .unpack::<StringValue>()
                .unwrap()
                .unwrap()
                .get_value(),
            "function_bar",
        );

        Ok(())
    }

    fn function_type_foo() -> FunctionType {
        FunctionType::new("namespace", "foo")
    }

    fn function_type_bar() -> FunctionType {
        FunctionType::new("namespace", "bar")
    }

    fn address_foo() -> Address {
        Address::new(function_type_foo(), "doctor")
    }

    fn address_bar() -> Address {
        Address::new(function_type_bar(), "doctor")
    }
}
