use crate::Serializable;
use std::marker::PhantomData;

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
pub struct TypeSpec<T> {
    pub(crate) typename: &'static str, // typename
    phantom: PhantomData<T>,
}

impl<T: Serializable + GetTypename> TypeSpec<T> {
    ///
    pub fn new() -> TypeSpec<T> {
        TypeSpec {
            typename: T::get_typename(),
            phantom: PhantomData,
        }
    }
}
