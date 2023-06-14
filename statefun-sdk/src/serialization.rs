// use protobuf::parse_from_bytes;
use crate::Serializable;
use protobuf::Message;
use statefun_proto::types::{
    BooleanWrapper, DoubleWrapper, FloatWrapper, IntWrapper, LongWrapper, StringWrapper,
};

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
        wrapped.set_value(*self);
        let res = wrapped.write_to_bytes().unwrap();

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
        wrapped.set_value(*self);
        let res = wrapped.write_to_bytes().unwrap();
        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<i64, String> {
        match LongWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value()),
            Err(result) => Err(result.to_string()),
        }
    }
}

impl Serializable<f32> for f32 {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = FloatWrapper::new();
        wrapped.set_value(*self);
        let res = wrapped.write_to_bytes().unwrap();
        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<f32, String> {
        match FloatWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value()),
            Err(result) => Err(result.to_string()),
        }
    }
}

impl Serializable<f64> for f64 {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = DoubleWrapper::new();
        wrapped.set_value(*self);
        let res = wrapped.write_to_bytes().unwrap();
        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<f64, String> {
        match DoubleWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value()),
            Err(result) => Err(result.to_string()),
        }
    }
}

impl Serializable<String> for String {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        let mut wrapped = StringWrapper::new();
        wrapped.set_value(self.clone());
        let res = wrapped.write_to_bytes().unwrap();

        Ok(res)
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<String, String> {
        match StringWrapper::parse_from_bytes(buffer) {
            Ok(result) => Ok(result.get_value().to_string()),
            Err(result) => Err(result.to_string()),
        }
    }
}
