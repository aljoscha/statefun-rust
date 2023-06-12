use std::fmt::{Display, Formatter};
use crate::{Serializable, BuiltInTypes, serializer, deserializer, ValueSpecBase};

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpec<T> {
    name : &'static str,  // state name
    pub (crate) typename : &'static str,  // type typename

    // todo: should these implement Result?
    pub (crate) serializer: fn(&T, String) -> Vec<u8>,
    pub (crate) deserializer: fn(String, &Vec<u8>) -> T,
}

impl<T: Serializable> ValueSpec<T> {
    // todo: could make this a trait by implementing as_const_str() on a static str
    ///
    pub const fn new(name: &'static str, built_in_type: BuiltInTypes) -> ValueSpec<T> {
        ValueSpec {
            name: name,
            typename: built_in_type.as_const_str(),
            serializer: serializer,
            deserializer: deserializer,
        }
    }

    ///
    pub const fn custom(name: &'static str, typename: &'static str) -> ValueSpec<T> {
        ValueSpec {
            name: name,
            typename: typename,
            serializer: serializer,
            deserializer: deserializer,
        }
    }

    fn as_base(&self) -> ValueSpecBase {
        ValueSpecBase {
            name : self.name.to_string(),
            typename : self.typename.to_string(),
        }
    }
}

///
impl<T> Into<ValueSpecBase> for ValueSpec<T> {
    ///
    fn into(self) -> ValueSpecBase {
        ValueSpecBase::new(self.name.to_string().as_str(), self.typename.to_string().as_str())
    }
}
