mod specs;
mod traits;
mod types;
use specs::*;
use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{
    Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType, Message,
};
use types::{EgressRecord, MyUserProfile, TotalVisitedUserIDs, UserLogin};

use statefun_greeter_example_proto::example::UserProfile;
use std::time::SystemTime;

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
                // user_login_spec().into(),
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
        if !message.is(&user_login_type_spec()) {
            panic!("Unexpected message type: {:?}", message.get_typed_value());
        }

        let user_login = match message.get::<UserLogin>() {
            Ok(user_login) => user_login,
            Err(error) => panic!("Could not receive UserLogin: {:?}", error),
        };

        log::info!("We should update user count {:?}", &user_login.user_name);

        let seen_count: Option<i32> = context.get_state(seen_count_spec());
        let seen_count = match seen_count {
            Some(count) => count + 1,
            None => 0,
        };

        let is_first_visit: bool = context.get_state(is_first_visit_spec()).is_none();

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
        effects
            .update_state(seen_count_spec(), &seen_count)
            .unwrap();
        effects
            .update_state(is_first_visit_spec(), &is_first_visit)
            .unwrap();
        effects
            .update_state(last_seen_timestamp_spec(), &last_seen_timestamp_ms)
            .unwrap();

        // let state_user_login: Option<UserLogin> = context.get_state(user_login_spec());
        // let state_user_login = match state_user_login {
        //     Some(existing_login) => existing_login,
        //     None => user_login,
        // };

        // log::info!("Seen user {:?} this many times: {:?}. Is this the first visit: {:?}. Timestamp of last visit: {:?}. User login: {:?}",
        //     &state_user_login.user_name, &seen_count, &is_first_visit,  &last_seen_timestamp_ms, &state_user_login);

        // effects.update_state(user_login_spec(), &state_user_login).unwrap();

        let mut profile = UserProfile::new();
        profile.set_name(user_login.user_name.to_string());
        profile.set_last_seen_delta_ms(last_seen_timestamp_ms);
        profile.set_login_location(format!("{:?}", user_login.login_type));
        profile.set_seen_count(seen_count);
        let profile = MyUserProfile(profile);

        effects
            .send(
                Address::new(greet_function(), &user_login.user_name.to_string()),
                user_profile_type_spec(),
                &profile,
            )
            .unwrap();

        effects
    }

    pub fn greet(_context: Context, message: Message) -> Effects {
        log::info!("--drey called greet: Received {:?}", &message);

        let user_profile: UserProfile = match message.get::<MyUserProfile>() {
            Ok(user_profile) => user_profile.0,
            Err(error) => panic!("Could not receive MyUserProfile: {:?}", error),
        };

        log::info!("We should greet {:?}", user_profile.get_name());

        let mut effects = Effects::new();
        let greetings = Self::create_greetings_message(user_profile);

        let egress_record = EgressRecord {
            topic: "greetings".to_string(),
            payload: greetings,
        };

        effects
            .egress(
                EgressIdentifier::new("io.statefun.playground", "egress"),
                egress_record_type_spec(),
                &egress_record,
            )
            .unwrap();

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
