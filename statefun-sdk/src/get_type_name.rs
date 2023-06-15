use crate::GetTypename;

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
