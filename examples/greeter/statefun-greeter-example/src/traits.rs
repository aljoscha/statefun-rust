use crate::{EgressRecord, MyUserProfile, UserLogin, UserProfile};
use protobuf::Message;
use statefun::{Serializable, TypeName};

impl Serializable<UserLogin> for UserLogin {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match serde_json::to_vec(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &[u8]) -> Result<UserLogin, String> {
        match serde_json::from_slice::<UserLogin>(buffer) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Serializable<MyUserProfile> for MyUserProfile {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match self.0.write_to_bytes() {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &[u8]) -> Result<MyUserProfile, String> {
        match UserProfile::parse_from_bytes(buffer) {
            Ok(result) => Ok(MyUserProfile(result)),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Serializable<EgressRecord> for EgressRecord {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match serde_json::to_vec(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &[u8]) -> Result<EgressRecord, String> {
        match serde_json::from_slice::<EgressRecord>(buffer) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl TypeName for UserLogin {
    ///
    fn get_typename() -> &'static str {
        "greeter.types/UserLogin"
    }
}

impl TypeName for MyUserProfile {
    ///
    fn get_typename() -> &'static str {
        "my-user-type/user-profile"
    }
}

impl TypeName for EgressRecord {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.playground/EgressRecord"
    }
}
