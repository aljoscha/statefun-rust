use serde::{Deserialize, Serialize};

use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{
    Address, BuiltInTypes, Context, Effects, FunctionRegistry, FunctionType, Serializable,
    StateMessage, ValueSpec, TypeName
};
use statefun_greeter_example_proto::example::UserProfile;
use std::time::SystemTime;
use protobuf::Message;

// todo: rename to TypeName
// todo: rename these BuiltInType values too, like Long => i64
const SEEN_COUNT: ValueSpec<i32> = ValueSpec::<i32>::new("seen_count", BuiltInTypes::Integer);
const IS_FIRST_VISIT: ValueSpec<bool> =
    ValueSpec::<bool>::new("is_first_visit", BuiltInTypes::Boolean);
const LAST_SEEN_TIMESTAMP: ValueSpec<i64> =
    ValueSpec::<i64>::new("last_seen_timestamp", BuiltInTypes::Long);
const USER_LOGIN: ValueSpec<UserLogin> =
    ValueSpec::<UserLogin>::custom("user_login", "my-user-type/user-login");

// only other way is to use lazy_static..
fn USER_FUNCTION() -> FunctionType {
    FunctionType::new("greeter.fns", "user")
}

fn GREET_FUNCTION() -> FunctionType {
    FunctionType::new("greeter.fns", "greet")
}

struct StatefulFunctions {
}

impl StatefulFunctions {
    pub fn new() -> StatefulFunctions {
        StatefulFunctions {
        }
    }

    pub fn register_functions(&self, function_registry: &mut FunctionRegistry) {
        function_registry.register_fn(
            USER_FUNCTION().clone(),
            vec![
                SEEN_COUNT.into(),
                IS_FIRST_VISIT.into(),
                LAST_SEEN_TIMESTAMP.into(),
                USER_LOGIN.into(),
            ],
            StatefulFunctions::user,
        );

        function_registry.register_fn(
            GREET_FUNCTION().clone(),
            vec![
                // todo: allow no state here too?
                // SEEN_COUNT.into(),
                // IS_FIRST_VISIT.into(),
                // LAST_SEEN_TIMESTAMP.into(),
                // USER_LOGIN.into(),
            ],
            StatefulFunctions::greet,
        );
    }

    // function_registry.register_fn(FunctionType::new("greeter.fns", "greetings"), vec![SEEN_COUNT.into()], greet);
    pub fn user(context: Context, message: StateMessage) -> Effects {
        let user_login = match message.get::<UserLogin>() {
            Some(user_login) => user_login,
            None => return Effects::new(),
        };

        log::info!("We should update user count {:?}", &user_login.user_name);

        let seen_count: Option<i32> = context.get_state(SEEN_COUNT);
        let seen_count = match seen_count {
            Some(count) => count + 1,
            None => 0,
        };

        let is_first_visit: Option<bool> = context.get_state(IS_FIRST_VISIT);
        let is_first_visit = match is_first_visit {
            Some(_) => false,
            None => true,
        };

        let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let last_seen_timestamp_ms: Option<i64> = context.get_state(LAST_SEEN_TIMESTAMP);
        let last_seen_timestamp_ms = match last_seen_timestamp_ms {
            Some(_) => current_time as i64,
            None => current_time as i64,
        };

        let mut effects = Effects::new();
        effects.update_state(SEEN_COUNT, &seen_count);
        effects.update_state(IS_FIRST_VISIT, &is_first_visit);
        effects.update_state(LAST_SEEN_TIMESTAMP, &last_seen_timestamp_ms);

        let state_user_login: Option<UserLogin> = context.get_state(USER_LOGIN);
        let state_user_login = match state_user_login {
            Some(existing_login) => existing_login,
            None => user_login,
        };

        log::info!("Seen user {:?} this many times: {:?}. Is this the first visit: {:?}. Timestamp of last visit: {:?}. User login: {:?}",
            &state_user_login.user_name, &seen_count, &is_first_visit,  &last_seen_timestamp_ms, &state_user_login);

        effects.update_state(USER_LOGIN, &state_user_login);

        let mut profile = UserProfile::new();
        profile.set_name(state_user_login.user_name.to_string());
        profile.set_last_seen_delta_ms(last_seen_timestamp_ms);
        profile.set_login_location(format!("{:?}", state_user_login.login_type));
        profile.set_seen_count(seen_count);
        let profile = MyUserProfile(profile);

        effects.send(
            Address::new(GREET_FUNCTION().clone(), &state_user_login.user_name.to_string()),
            USER_PROFILE_TYPE,
            &profile,
        );

        effects
    }

    pub fn greet(context: Context, message: StateMessage) -> Effects {
        log::info!("--drey called greet: Received {:?}", &message);
    //     // todo:
    //     // profile: UserProfile

    //     // log::info!("We should greet {:?}", profile.get_name());

        let mut effects = Effects::new();
    //     // let greetings = createGreetingsMessage(profile);

    //     // let mut egressRecord = EgressRecord::new();
    //     // egressRecord.set_topic("greetings".to_string());
    //     // egressRecord.set_payload(greetings);

    //     // effects.egress(EgressIdentifier::new("io.statefun.playground", "egress"),
    //     //                egressRecord);

        effects
    }

    // pub fn createGreetingsMessage(profile: UserProfile) -> String {
    //     let GREETINGS_TEMPLATES =
    //       ["Welcome", "Nice to see you again", "Third time is a charm"];

    //     let seenCount = profile.get_seen_count() as usize;

    //     if seenCount <= GREETINGS_TEMPLATES.len() {
    //       return format!("{:?} {:?}.", GREETINGS_TEMPLATES[seenCount], profile.get_name());
    //     } else {
    //       return format!(
    //         "Nice to see you for the {:?}th time, {:?}! It has been {:?} milliseconds since we last saw you.",
    //           seenCount, profile.get_name(), profile.get_last_seen_delta_ms());
    //     }
    // }

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
    WEB = 0,
    MOBILE = 1,
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
        let user_profile: UserProfile = UserProfile::parse_from_bytes(&buffer).unwrap();
        MyUserProfile(user_profile)
    }
}

const USER_PROFILE_TYPE : TypeName::<MyUserProfile> =
    TypeName::<MyUserProfile>::custom("my-user-type/user-profile");
