use statefun::io::kafka::KafkaEgress;
use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType, ValueSpecBase, ValueSpec, BuiltInTypes};
use statefun_greeter_example_proto::example::UserProfile;
use statefun_greeter_example_proto::example::EgressRecord;
use statefun_proto::request_reply::TypedValue;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

// todo: could we auto-convert here? E.g. to support u32 we could use i64 instead automatically?
// and then the BuiltInTypes doesn't have to be an enum.
fn SEEN_COUNT() -> ValueSpec<i32> {
    ValueSpec::<i32>::new("seen_count", BuiltInTypes::Integer)
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut function_registry = FunctionRegistry::new();
    // todo: need actual type here, either by doing `.withIntType()`, or by specifying our own namespace
    // todo: use namespaced type names here by making the namespace another parameter
    function_registry.register_fn(FunctionType::new("greeter.fns", "user"),      vec![SEEN_COUNT().into()], user);
    function_registry.register_fn(FunctionType::new("greeter.fns", "greetings"), vec![SEEN_COUNT().into()], greet);

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

pub fn user(context: Context, typed_value: TypedValue) -> Effects {
    let login: UserLogin = serde_json::from_slice(&typed_value.value).unwrap();

    log::info!("We should update user count {:?}", login.user_name);

    let seen_count: Option<i32> = context.get_state(SEEN_COUNT());
    let updated_seen_count = match seen_count {
        Some(count) => count + 1,
        None => 32, //
    };

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let now_ms = since_the_epoch.as_millis() as i64;

    // let last_seen_timestamp_ms : Option<Int64Value> = context.get_state("seen_timestamp_ms");
    // let mut updated_last_seen_timestamp_ms = match last_seen_timestamp_ms {
    //     Some(last_seen) => last_seen,
    //     None => { let mut x = Int64Value::new(); x.set_value(now_ms); x },
    // };

    let mut effects = Effects::new();
    // todo: store ValueSpec here
    effects.update_state(SEEN_COUNT(), &updated_seen_count);
    // effects.update_state("seen_timestamp_ms", &updated_last_seen_timestamp_ms);

    // log::info!(
    //     "We have seen {:?} {:?} times.",
    //     login.user_name,
    //     updated_seen_count
    // );

    // let mut profile = UserProfile::new();
    // profile.set_name(login.user_name.to_string());
    // profile.set_last_seen_delta_ms(now_ms);
    // profile.set_login_location(format!("{:?}", login.login_type));
    // profile.set_seen_count(updated_seen_count.value);

    // effects.send(
    //     Address::new(FunctionType::new("greeter.fns", "greetings"), &login.user_name),
    //     profile,
    // );

    effects
}

// todo: don't use TypedValue directly here
pub fn greet(_context: Context, typed_value: TypedValue) -> Effects {
    log::info!("--drey called greet: Received {:?}", typed_value);
    // todo:
    // profile: UserProfile

    // log::info!("We should greet {:?}", profile.get_name());

    let mut effects = Effects::new();
    // let greetings = createGreetingsMessage(profile);

    // let mut egressRecord = EgressRecord::new();
    // egressRecord.set_topic("greetings".to_string());
    // egressRecord.set_payload(greetings);

    // effects.egress(EgressIdentifier::new("io.statefun.playground", "egress"),
    //                egressRecord);

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
