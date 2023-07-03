use crate::{Expiration, Serializable, TypeName, ValueSpecBase};
use std::marker::PhantomData;

/// Defines the state of the function. Client code can use this type in the call to
/// `Context::get_state()` as a type-safe method of looking up existing state.
/// To pass a list of variadic `ValueSpec`'s to `FunctionRegistry::register_fn()` please
/// refer to the `specs![]` macro in the library.
// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
// #[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpec<T> {
    pub(crate) spec: ValueSpecBase,
    phantom: PhantomData<T>,
}

impl<T: Serializable<T> + TypeName> ValueSpec<T> {
    ///
    pub fn new(name: &'static str, expiration: Expiration) -> ValueSpec<T> {
        ValueSpec {
            spec: ValueSpecBase::new(name, T::get_typename(), expiration),
            phantom: PhantomData,
        }
    }
}

///
impl<T> From<ValueSpec<T>> for ValueSpecBase {
    ///
    fn from(val: ValueSpec<T>) -> Self {
        val.spec
    }
}
