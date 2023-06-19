use statefun::io::kafka::KafkaEgress;
use statefun::transport::hyper::HyperHttpTransport;
use statefun::transport::Transport;
use statefun_greeter_example_kafka_proto::example::GreetRequest;
use statefun_greeter_example_kafka_proto::example::GreetResponse;
use statefun::{
    Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType, Message, TypeName,
    Serializable, Expiration, ValueSpec, specs,
};
use protobuf::Message as ProtoMessage;

fn greeter_function_type() -> FunctionType {
    FunctionType::new("example", "greeter")
}

fn relay_function_type() -> FunctionType {
    FunctionType::new("example", "relay")
}

// 'seen_count' will automatically be purged 5 seconds after the last write
pub fn seen_count_spec() -> ValueSpec<i32> {
    ValueSpec::<i32>::new(
        "seen_count",
        Expiration::never(),
    )
}

/// Have to wrap the struct to implement traits
pub struct MyGreetRequest(pub GreetRequest);

impl TypeName for MyGreetRequest {
    ///
    fn get_typename() -> &'static str {
        "com.googleapis/example.GreetRequest"
    }
}

impl Serializable<MyGreetRequest> for MyGreetRequest {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match self.0.write_to_bytes() {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<MyGreetRequest, String> {
        match GreetRequest::parse_from_bytes(buffer) {
            Ok(result) => Ok(MyGreetRequest(result)),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub fn greet(context: Context, message: Message) -> Effects {
    if !message.is::<MyGreetRequest>() {
        panic!(
            "Unexpected message type: {:?}. Expected: {:?}",
            message.get_type(),
            MyGreetRequest::get_typename()
        );
    }

    let greet_request = match message.get::<MyGreetRequest>() {
        Ok(login) => login.0,
        Err(error) => panic!("Could not receive GreetRequest: {:?}", error),
    };

    log::info!("We should greet {:?}", greet_request.get_name());

    let seen_count = context.get_state(seen_count_spec());
        let seen_count = match seen_count {
            Some(count) => count.unwrap() + 1,
            None => 1,
        };

    log::info!(
        "We have seen {:?} {:?} times.",
        greet_request.get_name(),
        seen_count
    );

    let mut effects = Effects::new();

    effects.update_state(seen_count_spec(), &seen_count).unwrap();

    let mut greet_response = GreetResponse::new();
    greet_response.set_name(greet_request.get_name().to_owned());
    greet_response.set_greeting(format!(
        "Say hello to {} from Rust. I've seen them {} times now.",
        greet_request.get_name(),
        seen_count
    ));
    let my_greet_response = MyGreetResponse(greet_response);

    effects
        .send(
            Address::new(relay_function_type(), &greet_request.get_name()),
            &my_greet_response,
        )
        .unwrap();

    effects
}

/// Have to wrap the struct to implement traits
pub struct MyGreetResponse(pub GreetResponse);

impl TypeName for MyGreetResponse {
    ///
    fn get_typename() -> &'static str {
        "com.googleapis/example.GreetResponse"
    }
}

impl Serializable<MyGreetResponse> for MyGreetResponse {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match self.0.write_to_bytes() {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<MyGreetResponse, String> {
        match GreetResponse::parse_from_bytes(buffer) {
            Ok(result) => Ok(MyGreetResponse(result)),
            Err(error) => Err(error.to_string()),
        }
    }
}

pub fn relay(_context: Context, message: Message) -> Effects {
    let my_greetw = match message.get::<MyGreetResponse>() {
        Ok(my_greetw) => my_greetw,
        Err(error) => panic!("Could not receive GreetResponse: {:?}", error),
    };

    log::info!("Relaying message {:?} to Kafka.", message);

    let mut effects = Effects::new();

    effects.kafka_keyed_egress(
        EgressIdentifier::new("example", "greets"),
        "greetings",
        &my_greetw.0.get_name().to_string(),
        &my_greetw,
    ).unwrap();

    effects
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut function_registry = FunctionRegistry::new();

    function_registry.register_fn(
        greeter_function_type(),
        specs![seen_count_spec()],
        &greet,
    );

    function_registry.register_fn(
        relay_function_type(),
        vec![],
        &relay,
    );

    let hyper_transport = HyperHttpTransport::new("0.0.0.0:5000".parse()?);
    hyper_transport.run(function_registry)?;

    Ok(())
}
