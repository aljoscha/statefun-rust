use crate::{Serializable, GetTypename, ValueSpecBase};
use std::marker::PhantomData;

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpec<T> {
    name: &'static str,                // state name
    pub(crate) typename: &'static str, // type typename
    phantom: PhantomData<T>,
}

impl<T: Serializable + GetTypename> ValueSpec<T> {
    ///
    pub fn new(name: &'static str) -> ValueSpec<T> {
        ValueSpec {
            name,
            typename: T::get_typename(),
            phantom: PhantomData,
        }
    }
}

///
impl<T> From<ValueSpec<T>> for ValueSpecBase {
    ///
    fn from(val: ValueSpec<T>) -> Self {
        ValueSpecBase::new(
            val.name.to_string().as_str(),
            val.typename.to_string().as_str(),
        )
    }
}
