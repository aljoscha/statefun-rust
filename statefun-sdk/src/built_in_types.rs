///
pub enum BuiltInTypes {
    ///
    Boolean,
    ///
    Integer,
    ///
    Long,
    ///
    Float,
    ///
    Double,
    ///
    String,
}

impl BuiltInTypes {
    pub(crate) const fn as_const_str(&self) -> &'static str {
        match self {
            BuiltInTypes::Boolean => "io.statefun.types/bool",
            BuiltInTypes::Integer => "io.statefun.types/int",
            BuiltInTypes::Long => "io.statefun.types/long",
            BuiltInTypes::Float => "io.statefun.types/float",
            BuiltInTypes::Double => "io.statefun.types/double",
            BuiltInTypes::String => "io.statefun.types/string",
        }
    }
}

fn from_str(input: String) -> BuiltInTypes {
    match input.as_str() {
        "io.statefun.types/bool" => BuiltInTypes::Boolean,
        "io.statefun.types/int" => BuiltInTypes::Integer,
        "io.statefun.types/long" => BuiltInTypes::Long,
        "io.statefun.types/float" => BuiltInTypes::Float,
        "io.statefun.types/double" => BuiltInTypes::Double,
        "io.statefun.types/string" => BuiltInTypes::String,
        _ => panic!("Unexpected type"),
    }
}
