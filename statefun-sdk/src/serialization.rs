
///
pub trait Serializable {
    ///
    fn serialize(&self, typename: String) -> Vec<u8>;

    ///
    fn deserialize(typename: String, buffer: &Vec<u8>) -> Self;
}
