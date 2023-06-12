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
