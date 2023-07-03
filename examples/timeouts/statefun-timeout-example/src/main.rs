mod specs;
mod traits;
mod types;
use specs::*;
use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{
    specs, Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType, Message,
    TypeName,
};
use types::{DelayedMessage, EgressRecord, MyUserProfile, UserLogin};

use statefun_timeout_example_proto::example::UserProfile;
use std::time::Duration;
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
            is_first_visit_spec(),
            last_seen_timestamp_spec()
        ],
        user,
    );

    function_registry.register_fn(
        greet_function_type(),
        vec![], // no state
        greet,
    );

    function_registry.register_fn(
        delayed_function_type(),
        vec![], // no state
        delayed,
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

    let user_login = match message.get::<UserLogin>() {
        Ok(user_login) => user_login,
        Err(error) => panic!("Could not receive UserLogin: {:?}", error),
    };

    log::info!("We should update user count {:?}", &user_login.user_name);

    let seen_count = context.get_state(seen_count_spec());
    let seen_count = match seen_count {
        Some(count) => count.unwrap() + 1,
        None => 1,
    };

    let is_first_visit = context.get_state(is_first_visit_spec()).is_none();

    let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as i64,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    if let Some(result) = context.get_state(last_seen_timestamp_spec()) {
        log::info!("Last stored time: {:?}", result.unwrap())
    }

    log::info!("User login: {:?}. Seen user {:?} times. Is this the first visit: {:?}. Timestamp of last visit: {:?}.",
        &user_login, &seen_count, &is_first_visit, &current_time);

    let mut effects = Effects::new();

    // delete the profile every 5 visits, and send a delayed message to another function
    if seen_count >= 5 {
        effects.delete_state(seen_count_spec());
        effects.delete_state(is_first_visit_spec());

        let delayed_message = DelayedMessage::new(current_time);

        effects
            .send_after(
                Address::new(delayed_function_type(), &user_login.user_name),
                Duration::from_secs(3),
                "cancel-token".to_string(),
                &delayed_message,
            )
            .unwrap();
    } else {
        effects
            .update_state(seen_count_spec(), &seen_count)
            .unwrap();
        effects
            .update_state(is_first_visit_spec(), &is_first_visit)
            .unwrap();

        // cancel any pending message
        effects.cancel_delayed_message("cancel-token".to_string());
    }

    effects
        .update_state(last_seen_timestamp_spec(), &current_time)
        .unwrap();

    let mut profile = UserProfile::new();
    profile.set_name(user_login.user_name.to_string());
    profile.set_last_seen_delta_ms(current_time);
    profile.set_login_location(format!("{:?}", user_login.login_type));
    profile.set_seen_count(seen_count);
    let profile = MyUserProfile(profile);

    effects
        .send(
            Address::new(greet_function_type(), &user_login.user_name),
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

pub fn delayed(_context: Context, message: Message) -> Effects {
    let delayed_message = match message.get::<DelayedMessage>() {
        Ok(delayed_message) => delayed_message,
        Err(error) => panic!("Could not receive DelayedMessage: {:?}", error),
    };

    let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as i64,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    log::info!(
        "Received delayed message at {:?}, sent at {:?}",
        current_time,
        &delayed_message.time_sent
    );

    Effects::new()
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

fn delayed_function_type() -> FunctionType {
    FunctionType::new("greeter.fns", "delayed")
}
