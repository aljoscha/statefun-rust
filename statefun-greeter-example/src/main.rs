use serde::{Deserialize, Serialize};

use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{
    Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType,
    Serializable, Message, TypeSpec, ValueSpec, GetTypename,
};

use protobuf::Message as ProtoMessage;
use statefun_greeter_example_proto::example::UserProfile;
use std::time::SystemTime;

fn seen_count_spec() -> ValueSpec<i32> {
    ValueSpec::<i32>::new("seen_count")
}

fn is_first_visit_spec() -> ValueSpec<bool> {
    ValueSpec::<bool>::new("is_first_visit")
}

fn last_seen_timestamp_spec() -> ValueSpec<i64> {
    ValueSpec::<i64>::new("last_seen_timestamp")
}

fn user_login_spec() -> ValueSpec<UserLogin> {
    ValueSpec::<UserLogin>::new("user_login")
}

impl GetTypename for UserLogin {
    ///
    fn get_typename() -> &'static str {
        "my-user-type/user-login"
    }
}

// only other way is to use lazy_static..
fn user_function() -> FunctionType {
    FunctionType::new("greeter.fns", "user")
}

fn greet_function() -> FunctionType {
    FunctionType::new("greeter.fns", "greet")
}

struct StatefulFunctions {}

impl StatefulFunctions {
    pub fn new() -> StatefulFunctions {
        StatefulFunctions {}
    }

    pub fn register_functions(&self, function_registry: &mut FunctionRegistry) {
        function_registry.register_fn(
            user_function(),
            vec![
                seen_count_spec().into(),
                is_first_visit_spec().into(),
                last_seen_timestamp_spec().into(),
                user_login_spec().into(),
            ],
            StatefulFunctions::user,
        );

        function_registry.register_fn(
            greet_function(),
            vec![], // no state
            StatefulFunctions::greet,
        );
    }

    pub fn user(context: Context, message: Message) -> Effects {
        let user_login = match message.get::<UserLogin>() {
            Some(user_login) => user_login,
            None => return Effects::new(),
        };

        log::info!("We should update user count {:?}", &user_login.user_name);

        let seen_count: Option<i32> = context.get_state(seen_count_spec());
        let seen_count = match seen_count {
            Some(count) => count + 1,
            None => 0,
        };

        let is_first_visit: Option<bool> = context.get_state(is_first_visit_spec());
        let is_first_visit = is_first_visit.is_none();

        let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let last_seen_timestamp_ms: Option<i64> = context.get_state(last_seen_timestamp_spec());
        let last_seen_timestamp_ms = match last_seen_timestamp_ms {
            Some(_) => current_time as i64,
            None => current_time as i64,
        };

        let mut effects = Effects::new();
        effects.update_state(seen_count_spec(), &seen_count);
        effects.update_state(is_first_visit_spec(), &is_first_visit);
        effects.update_state(last_seen_timestamp_spec(), &last_seen_timestamp_ms);

        let state_user_login: Option<UserLogin> = context.get_state(user_login_spec());
        let state_user_login = match state_user_login {
            Some(existing_login) => existing_login,
            None => user_login,
        };

        log::info!("Seen user {:?} this many times: {:?}. Is this the first visit: {:?}. Timestamp of last visit: {:?}. User login: {:?}",
            &state_user_login.user_name, &seen_count, &is_first_visit,  &last_seen_timestamp_ms, &state_user_login);

        effects.update_state(user_login_spec(), &state_user_login);

        let mut profile = UserProfile::new();
        profile.set_name(state_user_login.user_name.to_string());
        profile.set_last_seen_delta_ms(last_seen_timestamp_ms);
        profile.set_login_location(format!("{:?}", state_user_login.login_type));
        profile.set_seen_count(seen_count);
        let profile = MyUserProfile(profile);

        effects.send(
            Address::new(greet_function(), &state_user_login.user_name.to_string()),
            user_profile_type_spec(),
            &profile,
        );

        effects
    }

    pub fn greet(_context: Context, message: Message) -> Effects {
        log::info!("--drey called greet: Received {:?}", &message);

        let user_profile: UserProfile = match message.get::<MyUserProfile>() {
            Some(user_profile) => user_profile.0,
            None => return Effects::new(), // todo: log
        };

        log::info!("We should greet {:?}", user_profile.get_name());

        let mut effects = Effects::new();
        let greetings = Self::create_greetings_message(user_profile);

        let egress_record = EgressRecord {
            topic: "greetings".to_string(),
            payload: greetings,
        };

        effects.egress(
            EgressIdentifier::new("io.statefun.playground", "egress"),
            egress_record_type_spec(),
            &egress_record,
        );

        effects
    }

    pub fn create_greetings_message(profile: UserProfile) -> String {
        let greetings_template = ["Welcome", "Nice to see you again", "Third time is a charm"];

        let seen_count = profile.get_seen_count() as usize;

        if seen_count <= greetings_template.len() {
            format!(
                "{:?} {:?}.",
                greetings_template[seen_count],
                profile.get_name()
            )
        } else {
            format!(
            "Nice to see you for the {:?}th time, {:?}! It has been {:?} milliseconds since we last saw you.",
              seen_count, profile.get_name(), profile.get_last_seen_delta_ms())
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let functions = StatefulFunctions::new();

    let mut function_registry = FunctionRegistry::new();
    functions.register_functions(&mut function_registry);
    let hyper_transport = HyperHttpTransport::new("0.0.0.0:1108".parse()?);
    hyper_transport.run(function_registry)?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
enum LoginType {
    Web = 0,
    Mobile = 1,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserLogin {
    user_id: String,
    user_name: String,
    login_type: LoginType,
}

// actual routines called by statefun SDK
impl Serializable for UserLogin {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> UserLogin {
        let login: UserLogin = serde_json::from_slice(buffer).unwrap();
        login
    }
}

// Have to wrap the struct to implement Serializable
struct MyUserProfile(UserProfile);

impl Serializable for MyUserProfile {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        self.0.write_to_bytes().unwrap()
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> MyUserProfile {
        let user_profile: UserProfile = UserProfile::parse_from_bytes(buffer).unwrap();
        MyUserProfile(user_profile)
    }
}

// A customized response sent to the user
#[derive(Serialize, Deserialize, Debug)]
struct EgressRecord {
    // The name of the user being greeted
    topic: String,

    // The users customized greeting
    payload: String,
}

impl Serializable for EgressRecord {
    fn serialize(&self, _typename: String) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> EgressRecord {
        let egress: EgressRecord = serde_json::from_slice(buffer).unwrap();
        egress
    }
}

fn user_profile_type_spec() -> TypeSpec<MyUserProfile> {
    TypeSpec::<MyUserProfile>::new()
}

impl GetTypename for MyUserProfile {
    ///
    fn get_typename() -> &'static str {
        "my-user-type/user-profile"
    }
}

// note: the playground image actually hardcodes this check so we have to match it for now,
// until we configure our own playground
fn egress_record_type_spec() -> TypeSpec<EgressRecord> {
    TypeSpec::<EgressRecord>::new()
}

impl GetTypename for EgressRecord {
    ///
    fn get_typename() -> &'static str {
        "io.statefun.playground/EgressRecord"
    }
}
