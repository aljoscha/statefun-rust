use crate::{deserializer, serializer, Serializable /*, ValueSpecBase*/};

///
pub trait GetTypename {
    ///
    fn get_typename() -> &'static str;
}

impl GetTypename for bool {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/bool"
    }
}

impl GetTypename for i32 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/int"
    }
}

impl GetTypename for i64 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/long"
    }
}

impl GetTypename for f32 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/float"
    }
}

impl GetTypename for f64 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/double"
    }
}

impl GetTypename for String {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/string"
    }
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct TypeName<T> {
    pub(crate) typename: &'static str, // typename

    // todo: should these implement Result?
    pub(crate) serializer: fn(&T, String) -> Vec<u8>,
    pub(crate) deserializer: fn(String, &Vec<u8>) -> T,
}

impl<T: Serializable + GetTypename> TypeName<T> {
    ///
    pub fn new() -> TypeName<T> {
        TypeName {
            typename: T::get_typename(),
            serializer,
            deserializer,
        }
    }
}
