//! The function registry keeps a mapping from `FunctionType` to stateful functions.

use std::collections::HashMap;

use protobuf::well_known_types::Any;
use protobuf::Message;

use crate::InvocationError::FunctionNotFound;
use crate::{Context, Effects, FunctionType, InvocationError};
use crate::MissingStateCollection;
use crate::ValueSpec;

use statefun_proto::request_reply::TypedValue;

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
    pub fn register_fn<F: Fn(Context, TypedValue) -> Effects + Send + 'static>(
        &mut self,
        function_type: FunctionType,
        value_specs: Vec<ValueSpec>,
        function: F,
    ) {
        let callable_function = FnInvokableFunction {
            function,
            marker: ::std::marker::PhantomData,
            value_specs
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
        message: TypedValue,
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
    fn invoke(&self, context: Context, message: TypedValue) -> Result<Effects, InvocationError>;
}

/// An `InvokableFunction` that is backed by a `Fn`.
struct FnInvokableFunction<F: Fn(Context, TypedValue) -> Effects> {
    function: F,
    marker: ::std::marker::PhantomData<TypedValue>,
    // todo: these should be specc'ed out like TypeName in the Java SDK,
    // for now we're just storing plain strings w/o any validation.
    value_specs: Vec<ValueSpec>,
}

impl<F: Fn(Context, TypedValue) -> Effects> InvokableFunction for FnInvokableFunction<F> {
    fn invoke(&self, context: Context, message: TypedValue) -> Result<Effects, InvocationError> {

        let mut missing_states : Vec<ValueSpec> = Vec::new();
        for value_spec in (&self.value_specs).into_iter() {
            if !context.state.contains_key(&value_spec.name) {
                missing_states.push(value_spec.clone());
            }
        }

        if missing_states.len() > 0 {
            return Err(InvocationError::MissingStates(MissingStateCollection { states: missing_states }));
        }

        log::debug!("--drey: Trying to unpack message: {:?}", &message);
        let effects = (self.function)(context, message);
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

        // todo: fixup with invoke
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

        // todo: Fixup
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
        // todo: fixup
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
        // todo: fixup
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
