use crate::{deserializer, serializer, BuiltInTypes, Serializable/*, ValueSpecBase*/};

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct TypeName<T> {
    pub(crate) typename: &'static str, // typename

    // todo: should these implement Result?
    pub(crate) serializer: fn(&T, String) -> Vec<u8>,
    pub(crate) deserializer: fn(String, &Vec<u8>) -> T,
}

impl<T: Serializable> TypeName<T> {
    // todo: could make this a trait by implementing as_const_str() on a static str
    ///
    pub const fn new(built_in_type: BuiltInTypes) -> TypeName<T> {
        TypeName {
            typename: built_in_type.as_const_str(),
            serializer,
            deserializer,
        }
    }

    ///
    pub const fn custom(typename: &'static str) -> TypeName<T> {
        TypeName {
            typename,
            serializer,
            deserializer,
        }
    }
}

// ///
// impl<T> From<TypeName<T>> for ValueSpecBase {
//     ///
//     fn from(val: TypeName<T>) -> Self {
//         ValueSpecBase::new(
//             val.name.to_string().as_str(),
//             val.typename.to_string().as_str(),
//         )
//     }
// }
