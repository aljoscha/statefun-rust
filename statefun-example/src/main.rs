use exitfailure::ExitFailure;

use statefun_example_protos::example::GreetRequest;
use statefun_example_protos::example::GreetResponse;
use statefun_sdk::functions::{Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType};
use statefun_sdk::transport::hyper::HyperHttpTransport;
use statefun_sdk::transport::Transport;

pub fn greet(_context: Context, request: GreetRequest) -> Effects {
    log::info!("We should greet {:?}", request.get_who());

    let mut effects = Effects::new();

    let mut greet_response = GreetResponse::new();
    greet_response.set_who(request.get_who().to_owned());
    greet_response.set_greeting("ciao".to_string());

    effects.egress(
        EgressIdentifier::new("org.apache.flink.statefun.examples.harness", "out"),
        greet_response,
    );

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
