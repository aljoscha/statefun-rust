mod specs;
mod traits;
mod types;
use specs::*;
use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{
    Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType, Message, TypeName,
    specs,
};
use types::{EgressRecord, MyUserProfile, UserLogin};

use statefun_greeter_example_proto::example::UserProfile;
use std::time::SystemTime;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut function_registry = FunctionRegistry::new();
    register_functions(&mut function_registry);

    let hyper_transport = HyperHttpTransport::new("0.0.0.0:1108".parse()?);
    hyper_transport.run(function_registry)?;

    Ok(())
}

pub fn register_functions(function_registry: &mut FunctionRegistry) {
    function_registry.register_fn(
        user_function_type(),
        specs![
            seen_count_spec(),
            last_seen_timestamp_spec()
        ],
        user,
    );

    function_registry.register_fn(
        greet_function_type(),
        vec![], // no state
        greet,
    );
}

pub fn user(context: Context, message: Message) -> Effects {
    if !message.is::<UserLogin>() {
        panic!(
            "Unexpected message type: {:?}. Expected: {:?}",
            message.get_type(),
            UserLogin::get_typename()
        );
    }

    let login = match message.get::<UserLogin>() {
        Ok(login) => login,
        Err(error) => panic!("Could not receive UserLogin: {:?}", error),
    };

    let seen_count = context.get_state(seen_count_spec());
    let seen_count = match seen_count {
        Some(count) => count.unwrap() + 1,
        None => 1,
    };

    let now_ms = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as i64,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    let last_seen_timestamp_ms = match context.get_state(last_seen_timestamp_spec()) {
        Some(seen_ms) => seen_ms.unwrap(),
        None => now_ms,
    };

    let mut effects = Effects::new();
    effects.update_state(seen_count_spec(), &seen_count).unwrap();
    effects.update_state(last_seen_timestamp_spec(), &now_ms).unwrap();


    let mut profile = UserProfile::new();
    profile.set_name(login.user_name.to_string());
    profile.set_login_location(format!("{:?}", login.login_type));
    profile.set_seen_count(seen_count);
    profile.set_last_seen_delta_ms(now_ms - last_seen_timestamp_ms);
    let profile = MyUserProfile(profile);

    effects
        .send(
            Address::new(greet_function_type(), &login.user_name),
            &profile,
        )
        .unwrap();

    effects
}

pub fn greet(_context: Context, message: Message) -> Effects {
    let user_profile = match message.get::<MyUserProfile>() {
        Ok(user_profile) => user_profile.0,
        Err(error) => panic!("Could not receive MyUserProfile: {:?}", error),
    };

    log::info!("We should greet {:?}", user_profile.get_name());

    let mut effects = Effects::new();
    let greetings = create_greetings_message(user_profile);

    let egress_record = EgressRecord {
        topic: "greetings".to_string(),
        payload: greetings,
    };

    effects
        .egress(
            EgressIdentifier::new("io.statefun.playground", "egress"),
            &egress_record,
        )
        .unwrap();

    effects
}

fn create_greetings_message(profile: UserProfile) -> String {
    let greetings_template = ["Welcome", "Nice to see you again", "Third time is a charm"];

    let seen_count = profile.get_seen_count() as usize;

    if seen_count <= greetings_template.len() {
        format!(
            "{:?} {:?}.",
            greetings_template[seen_count - 1],
            profile.get_name()
        )
    } else {
        format!(
        "Nice to see you for the {:?}th time, {:?}! It has been {:?} milliseconds since we last saw you.",
          seen_count, profile.get_name(), profile.get_last_seen_delta_ms())
    }
}

// lazy_static does not work here for some reason
fn user_function_type() -> FunctionType {
    FunctionType::new("greeter.fns", "user")
}

fn greet_function_type() -> FunctionType {
    FunctionType::new("greeter.fns", "greet")
}
