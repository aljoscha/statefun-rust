use crate::{deserializer, serializer, Serializable, GetTypename, ValueSpecBase};

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpec<T> {
    name: &'static str,                // state name
    pub(crate) typename: &'static str, // type typename

    // todo: should these implement Result?
    pub(crate) serializer: fn(&T, String) -> Vec<u8>,
    pub(crate) deserializer: fn(String, &Vec<u8>) -> T,
}

impl<T: Serializable + GetTypename> ValueSpec<T> {
    ///
    pub fn new(name: &'static str) -> ValueSpec<T> {
        ValueSpec {
            name,
            typename: T::get_typename(),
            serializer,
            deserializer,
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
