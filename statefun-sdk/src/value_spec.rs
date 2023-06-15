use crate::{TypeName, Serializable, Expiration, ValueSpecBase};
use std::marker::PhantomData;

///
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
