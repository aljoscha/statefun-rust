use protobuf::well_known_types::Int32Value;

use statefun::io::kafka::KafkaEgress;
use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun::{Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType};
use statefun_example_proto::example::GreetRequest;
use statefun_example_proto::example::GreetResponse;

pub fn greet(context: Context, request: GreetRequest) -> Effects {
    log::info!("We should greet {:?}", request.get_name());

    let seen_count: Option<Int32Value> = context.get_state("seen_count");

    log::info!(
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

    effects.send(
        Address::new(FunctionType::new("example", "relay"), request.get_name()),
        greet_response,
    );

    effects
}

pub fn relay(_context: Context, message: GreetResponse) -> Effects {
    log::info!("Relaying message {:?} to Kafka.", message);

    let mut effects = Effects::new();

    effects.kafka_keyed_egress(
        EgressIdentifier::new("example", "greets"),
        "greeting",
        &message.get_name().to_string(),
        message,
    );

    effects
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut function_registry = FunctionRegistry::new();
    function_registry.register_fn(FunctionType::new("example", "greeter"), greet);
    function_registry.register_fn(FunctionType::new("example", "relay"), relay);

    let hyper_transport = HyperHttpTransport::new("0.0.0.0:5000".parse()?);
    hyper_transport.run(function_registry)?;

    Ok(())
}
