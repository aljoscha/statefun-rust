use exitfailure::ExitFailure;

use protobuf::well_known_types::Int32Value;
use statefun_example_protos::example::GreetRequest;
use statefun_example_protos::example::GreetResponse;
use statefun_sdk::io::kafka;
use statefun_sdk::transport::hyper::HyperHttpTransport;
use statefun_sdk::transport::Transport;
use statefun_sdk::{Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType};

pub fn greet(context: Context, request: GreetRequest) -> Effects {
    log::debug!("We should greet {:?}", request.get_name());

    let seen_count: Option<Int32Value> = context.get_state("seen_count");

    log::debug!(
        "We have seen {:?} {:?} times.",
        request.get_name(),
        seen_count
    );

    let mut effects = Effects::new();

    let mut updated_seen_count = match seen_count {
        Some(count) => count,
        None => Int32Value::new(),
    };

    updated_seen_count.set_value(updated_seen_count.get_value() + 1);

    effects.update_state("seen_count", &updated_seen_count);

    let mut greet_response = GreetResponse::new();
    greet_response.set_name(request.get_name().to_owned());
    greet_response.set_greeting(format!(
        "Say hello to {} from Rust. I've seen them {} times now.",
        request.get_name(),
        updated_seen_count.get_value()
    ));
    let kafka_message = kafka::keyed_egress_record("greetings", request.get_name(), greet_response);
    effects.egress(EgressIdentifier::new("example", "greets"), kafka_message);

    effects
}

fn main() -> Result<(), ExitFailure> {
    env_logger::init();

    let mut function_registry = FunctionRegistry::new();
    function_registry.register_fn(FunctionType::new("example", "greeter"), greet);

    let hyper_transport = HyperHttpTransport::new("127.0.0.1:5000".parse()?);
    hyper_transport.run(function_registry)?;

    Ok(())
}
