use serde::{Deserialize, Serialize};
use statefun_timeout_example_proto::example::UserProfile;

///
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")] // note: using uppercase in the HTTP request
pub enum LoginType {
    Web = 0,
    Mobile = 1,
}

///
#[derive(Serialize, Deserialize, Debug)]
pub struct UserLogin {
    pub user_id: String,
    pub user_name: String,
    pub login_type: LoginType,
}

///
#[derive(Serialize, Deserialize, Debug)]
pub struct DelayedMessage {
    pub time_sent: i64,
}

impl DelayedMessage {
    pub fn new(time_sent: i64) -> DelayedMessage {
        DelayedMessage { time_sent }
    }
}

/// A customized response sent to the user
#[derive(Serialize, Deserialize, Debug)]
pub struct EgressRecord {
    // The name of the user being greeted
    pub topic: String,

    // The users customized greeting
    pub payload: String,
}

/// Have to wrap the struct to implement Serializable
pub struct MyUserProfile(pub UserProfile);
