use statefun::{
    GetTypename, Serializable,
};
use crate::{EgressRecord, UserLogin, TotalVisitedUserIDs, MyUserProfile, UserProfile};
use protobuf::Message;

impl Serializable<TotalVisitedUserIDs> for TotalVisitedUserIDs {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match serde_json::to_vec(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<TotalVisitedUserIDs, String> {
        match serde_json::from_slice::<TotalVisitedUserIDs>(buffer) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Serializable<UserLogin> for UserLogin {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match serde_json::to_vec(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<UserLogin, String> {
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

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<MyUserProfile, String> {
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

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<EgressRecord, String> {
        match serde_json::from_slice::<EgressRecord>(buffer) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl GetTypename for UserLogin {
    ///
    fn get_typename() -> &'static str {
        "my-user-type/user-login"
    }
}

impl GetTypename for TotalVisitedUserIDs {
    ///
    fn get_typename() -> &'static str {
        "my-user-type/total-visited-ids"
    }
}

impl GetTypename for MyUserProfile {
    ///
    fn get_typename() -> &'static str {
        "my-user-type/user-profile"
    }
}

impl GetTypename for EgressRecord {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.playground/EgressRecord"
    }
}
