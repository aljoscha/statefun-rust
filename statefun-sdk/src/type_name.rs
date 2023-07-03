use crate::TypeName;

impl TypeName for bool {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/bool"
    }
}

impl TypeName for i32 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/int"
    }
}

impl TypeName for i64 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/long"
    }
}

impl TypeName for f32 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/float"
    }
}

impl TypeName for f64 {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/double"
    }
}

impl TypeName for String {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.types/string"
    }
}
