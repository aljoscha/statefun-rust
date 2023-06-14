
///
pub trait GetTypename {
    ///
    fn get_typename() -> &'static str;
}

///
pub trait Serializable<T> {
    ///
    fn serialize(&self, typename: String) -> Result<Vec<u8>, String>;

    ///
    fn deserialize(typename: String, buffer: &Vec<u8>) -> Result<T, String>;
}
