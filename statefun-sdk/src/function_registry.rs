//! The function registry keeps a mapping from `FunctionType` to stateful functions.

use std::collections::HashMap;

use crate::InvocationError::FunctionNotFound;
use crate::Message;
use crate::MissingStates;
use crate::ValueSpecBase;
use crate::{Context, Effects, FunctionType, InvocationError};

// use statefun_proto::request_reply::TypedValue;

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
    pub fn register_fn<F: Fn(Context, Message) -> Effects + Send + 'static>(
        &mut self,
        function_type: FunctionType,
        value_specs: Vec<ValueSpecBase>,
        function: F,
    ) {
        let callable_function = FnInvokableFunction {
            function,
            marker: ::std::marker::PhantomData,
            value_specs,
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
        message: Message,
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
    fn invoke(&self, context: Context, message: Message) -> Result<Effects, InvocationError>;
}

/// An `InvokableFunction` that is backed by a `Fn`.
struct FnInvokableFunction<F: Fn(Context, Message) -> Effects> {
    function: F,
    marker: ::std::marker::PhantomData<Message>,
    value_specs: Vec<ValueSpecBase>,
}

impl<F: Fn(Context, Message) -> Effects> InvokableFunction for FnInvokableFunction<F> {
    fn invoke(&self, context: Context, message: Message) -> Result<Effects, InvocationError> {
        let mut missing_states: Vec<ValueSpecBase> = Vec::new();

        // NOTE: The API is very tricky:
        //
        // Context for a function's state can be in one of three states:
        // A) Missing, for example when this is a brand new state variable Flink doesn't know about.
        // B) Allocated but uninitialized, when Flink allocates storage for this state variable
        //    but doesn't have any value stored in it yet.
        // C) Allocated and initialized, when a function has stored a value in a state variable
        //    successfully (this means Flink received the response for a state mutation).
        //
        // In each of these three cases Flink sends wildly different `ToFunction.PersistedValue`
        // in the request.
        //
        // - Assume a new state value called `my_state` that stores an `i32`
        // - When a state value is first introduced in a function, in the first call the context
        //   will not contain this state value. We return `incomplete_invocation_context` to let
        //   Flink allocate storage for this state.
        // - Flink then prepares storage for `my_state` and calls the function again.
        //   The context will contain `ValueSpecBase { name: "my_state", typename: "" }: []`
        //   Note how the `typename` is still empty here despite it being set in the previous
        //   `incomplete_invocation_context` response. This could be a Flink Statefun bug..
        // - Afterwards when we initialize this state to a value, e.g. 42, context will contain:
        //   `ValueSpecBase { name: "my_state", typename: "io.statefun.types/int" }: [0x42]`
        //
        // - Therefore we cannot check the typename consistently as it's only ever set after the
        //   first time we write to the state.
        //
        // See also:
        //   - https://issues.apache.org/jira/browse/FLINK-20265
        //   - https://github.com/apache/flink-statefun/pull/177

        for value_spec in self.value_specs.iter() {
            let mut found: bool = false;
            for context_spec in context.state.iter() {
                if value_spec.name.eq(&context_spec.0.name) {
                    found = true;
                    break;
                }
            }

            if !found {
                missing_states.push(value_spec.clone());
            }
        }

        if !missing_states.is_empty() {
            return Err(InvocationError::MissingStates(MissingStates {
                states: missing_states,
            }));
        }

        let effects = (self.function)(context, message);
        Ok(effects)
    }
}

#[cfg(test)]
mod tests {
    use protobuf::well_known_types::StringValue;
    // use crate::function_registry::HashMap;
    use crate::FunctionRegistry;
    use crate::*;
    use protobuf::Message as ProtoMessage;
    use std::collections::HashMap;

    fn to_typed_value(typename: String, value: Vec<u8>) -> TypedValue {
        let mut res = TypedValue::new();
        res.set_typename(typename);
        res.set_has_value(true);
        res.set_value(value);
        res
    }

    #[test]
    fn call_registered_function() -> anyhow::Result<()> {
        let state = HashMap::new();
        let address = address_foo().into_proto();
        let context = Context::new(&state, &address, &address);

        let mut registry = FunctionRegistry::new();
        registry.register_fn(
            function_type_foo(),
            vec![],
            |_context, _message: Message| Effects::new(),
        );

        let message = Message::new(to_typed_value("some-type".to_string(), vec![]));
        let _effects = registry.invoke(function_type_foo(), context, message)?;

        Ok(())
    }

    #[test]
    fn call_unknown_function() -> anyhow::Result<()> {
        let state = HashMap::new();
        let address = address_foo().into_proto();
        let context = Context::new(&state, &address, &address);

        let registry = FunctionRegistry::new();
        let message = Message::new(to_typed_value("some-type".to_string(), vec![]));

        let result = registry.invoke(function_type_bar(), context, message);
        assert!(result.is_err());

        Ok(())
    }

    /// Have to wrap the struct to implement Serializable
    pub struct MyStringValue(pub StringValue);

    impl Serializable<MyStringValue> for MyStringValue {
        fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
            match self.0.write_to_bytes() {
                Ok(result) => Ok(result),
                Err(error) => Err(error.to_string()),
            }
        }

        fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<MyStringValue, String> {
            match StringValue::parse_from_bytes(buffer) {
                Ok(result) => Ok(MyStringValue(result)),
                Err(error) => Err(error.to_string()),
            }
        }
    }

    impl TypeName for MyStringValue {
        ///
        fn get_typename() -> &'static str {
            "my-string-value/string-value"
        }
    }

    #[test]
    fn call_correct_function() -> anyhow::Result<()> {
        let state = HashMap::new();

        let mut registry = FunctionRegistry::new();
        registry.register_fn(function_type_foo(), vec![], |context, _message: Message| {
            let mut effects = Effects::new();

            let mut message = StringValue::new();
            message.set_value("function_foo".to_owned());
            let message = MyStringValue(message);
            effects.send(context.self_address(), &message).unwrap();

            effects
        });

        registry.register_fn(function_type_bar(), vec![], |context, _message: Message| {
            let mut effects = Effects::new();

            let mut message = StringValue::new();
            message.set_value("function_bar".to_owned());
            let message = MyStringValue(message);
            effects.send(context.self_address(), &message).unwrap();

            effects
        });

        let address_foo = address_foo().into_proto();
        let context = Context::new(&state, &address_foo, &address_foo);
        let message = Message::new(to_typed_value("some-type".to_string(), vec![]));
        let effects_foo = registry.invoke(function_type_foo(), context, message)?;
        assert_eq!(
            MyStringValue::deserialize("some-type".to_string(), &effects_foo.invocations[0].2)
                .unwrap()
                .0
                .value,
            "function_foo",
        );

        let address_bar = address_bar().into_proto();
        let context = Context::new(&state, &address_bar, &address_bar);
        let message = Message::new(to_typed_value("some-type".to_string(), vec![]));
        let effects_bar = registry.invoke(function_type_bar(), context, message)?;
        assert_eq!(
            MyStringValue::deserialize("some-type".to_string(), &effects_bar.invocations[0].2)
                .unwrap()
                .0
                .value,
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
