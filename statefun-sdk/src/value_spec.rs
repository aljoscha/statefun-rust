use crate::{deserializer, serializer, BuiltInTypes, Serializable, ValueSpecBase};


///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpec<T> {
    name: &'static str,                // state name
    pub(crate) typename: &'static str, // type typename

    // todo: should these implement Result?
    pub(crate) serializer: fn(&T, String) -> Vec<u8>,
    pub(crate) deserializer: fn(String, &Vec<u8>) -> T,
}

impl<T: Serializable> ValueSpec<T> {
    // todo: could make this a trait by implementing as_const_str() on a static str
    ///
    pub const fn new(name: &'static str, built_in_type: BuiltInTypes) -> ValueSpec<T> {
        ValueSpec {
            name,
            typename: built_in_type.as_const_str(),
            serializer,
            deserializer,
        }
    }

    ///
    pub const fn custom(name: &'static str, typename: &'static str) -> ValueSpec<T> {
        ValueSpec {
            name,
            typename,
            serializer,
            deserializer,
        }
    }

    fn as_base(&self) -> ValueSpecBase {
        ValueSpecBase {
            name: self.name.to_string(),
            typename: self.typename.to_string(),
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
