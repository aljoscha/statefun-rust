// use protobuf::parse_from_bytes;
use protobuf::Message;
use statefun_proto::types::{BooleanWrapper, IntWrapper, LongWrapper, StringWrapper};
use crate::Serializable;

impl Serializable<bool> for bool {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = BooleanWrapper::new();
        wrapped.set_value(*self);
        match wrapped.write_to_bytes() {
            Ok(result) => Ok(result),
            Err(result) => Err(result.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<bool, String> {
        match BooleanWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value()),
            Err(result) => Err(result.to_string()),
        }
    }
}

impl Serializable<i32> for i32 {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = IntWrapper::new();
        log::debug!("-- drey: i32 serializing {:?}", self);
        wrapped.set_value(*self);
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<i32, String> {
        match IntWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value()),
            Err(result) => Err(result.to_string()),
        }
    }
}

impl Serializable<i64> for i64 {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = LongWrapper::new();
        log::debug!("-- drey: i64 serializing {:?}", self);
        wrapped.set_value(*self);
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<i64, String> {
        match LongWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value()),
            Err(result) => Err(result.to_string()),
        }
    }
}

impl Serializable<String> for String {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = StringWrapper::new();
        log::debug!("-- drey: String serializing {:?}", self);
        wrapped.set_value(self.clone());
        log::debug!("-- drey: wrapped {:?}", wrapped);
        let res = wrapped.write_to_bytes().unwrap();
        log::debug!("-- drey: res {:?}", res);

        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<String, String> {
        match StringWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value().to_string()),
            Err(result) => Err(result.to_string()),
        }
    }
}
