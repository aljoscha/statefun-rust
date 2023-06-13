// use protobuf::parse_from_bytes;
use protobuf::Message;
use statefun_proto::types::{BooleanWrapper, IntWrapper, LongWrapper, StringWrapper};

///
pub trait Serializable {
    ///
    fn serialize(&self, typename: String) -> Vec<u8>;

    ///
    fn deserialize(typename: String, buffer: &Vec<u8>) -> Self;
}

impl Serializable for bool {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        let mut wrapped = BooleanWrapper::new();
        wrapped.set_value(*self);
        wrapped.write_to_bytes().unwrap()
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> bool {
        let wrapped = BooleanWrapper::parse_from_bytes(buffer).unwrap();
        wrapped.get_value()
    }
}

impl Serializable for i32 {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        let mut wrapped = IntWrapper::new();
        log::debug!("-- drey: i32 serializing {:?}", self);
        wrapped.set_value(*self);
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        res
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> i32 {
        let wrapped = IntWrapper::parse_from_bytes(buffer).unwrap();
        wrapped.get_value()
    }
}

impl Serializable for i64 {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        let mut wrapped = LongWrapper::new();
        log::debug!("-- drey: i64 serializing {:?}", self);
        wrapped.set_value(*self);
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        res
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> i64 {
        let wrapped = LongWrapper::parse_from_bytes(buffer).unwrap();
        wrapped.get_value()
    }
}

impl Serializable for String {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        let mut wrapped = StringWrapper::new();
        log::debug!("-- drey: String serializing {:?}", self);
        wrapped.set_value(self.clone());
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        res
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> String {
        let wrapped = StringWrapper::parse_from_bytes(buffer).unwrap();
        wrapped.get_value().to_string()
    }
}

pub(crate) fn serializer<T: Serializable>(value: &T, typename: String) -> Vec<u8> {
    // log::debug!("-- drey: serializing type: {:?}", typename);
    value.serialize(typename)
    // log::debug!("-- drey: serialized to: {:?}", &res);
}

// todo
pub(crate) fn deserializer<T: Serializable>(typename: String, buffer: &Vec<u8>) -> T {
    // log::debug!("-- drey: deserializing type: {:?}", typename);
    // todo: how do we limit T here so T::new will work??
    // T::new()
    // panic!("oops")

    T::deserialize(typename, buffer)
}
