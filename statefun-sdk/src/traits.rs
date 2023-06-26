/// Each message type must implement this trait, which returns the fully qualified type name of
/// this type. For example, for native integers the SDK provides an implementation of this trait
/// which returns "io.statefun.types/bool".
pub trait TypeName {
    /// Returns the fully qualified name of this type, for example "com.my.company/user-type"
    fn get_typename() -> &'static str;
}

/// Each message type must implement this trait, which allows the client code to define how the
/// the type is serialized & deserialized. Some types are supported by the SDK natively,
/// which are serialized to Protobuf under the hood. This allows built-in types like booleans,
/// integers, floating point, and strings to be serialized in a common cross-language compatible
/// format.
pub trait Serializable<T> {
    /// Implements serialization
    fn serialize(&self, typename: String) -> Result<Vec<u8>, String>;

    /// Implements deserialization
    fn deserialize(typename: String, buffer: &Vec<u8>) -> Result<T, String>;
}
