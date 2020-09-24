use futures::stream::StreamExt;
use protobuf::Message;
use rand::seq::SliceRandom;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::Message as KafkaMessage;
use structopt::StructOpt;

use statefun_example_proto::example::GreetRequest;
use statefun_example_proto::example::GreetResponse;

#[derive(StructOpt, Debug)]
#[structopt(name = "datagen")]
struct Opt {
    /// Broker list in kafka format
    #[structopt(short, long, default_value = "kafka-broker:9092")]
    brokers: String,

    /// Consumer group id
    #[structopt(short, long, default_value = "datagen-group")]
    group_id: String,

    /// Topic that we produce to
    #[structopt(short, long, default_value = "names")]
    output_topic: String,

    /// Topic that we consume from
    #[structopt(short, long, default_value = "greetings")]
    input_topic: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opt = Opt::from_args();
    log::debug!("Commandline options: {:?}", opt);

    let producer = tokio::spawn(run_producer(opt.brokers.clone(), opt.output_topic.clone()));

    let consumer = tokio::spawn(run_consumer(
        opt.brokers.clone(),
        opt.group_id.clone(),
        opt.input_topic.clone(),
    ));

    let (producer_result, consumer_result) = tokio::join!(producer, consumer);

    // Just let it panic. Kids, don't do this at home!
    producer_result.unwrap();
    consumer_result.unwrap();

    Ok(())
}

async fn run_producer(brokers: String, topic: String) {
    let names = ["Leto", "Paul", "Jessica", "Duncan", "Gurney"];

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .create()
        .expect("Producer creation error");

    loop {
        // we cannot cache the rng in a variable because a) futures are not tied to threads and
        // this is a thread-local rnc b) the resulting Future of this async fn would not be Send,
        // the compiler is protecting us from a) here.
        let name = names.choose(&mut rand::thread_rng()).unwrap();

        let mut greet_request = GreetRequest::new();
        greet_request.set_name(name.to_string());

        let serialized_request = greet_request.write_to_bytes().unwrap();

        producer
            .send_result(
                FutureRecord::to(&topic)
                    .key(name.to_owned())
                    .payload(&serialized_request),
            )
            .expect("Sending failed")
            .await
            .unwrap()
            .unwrap();
    }
}

async fn run_consumer(brokers: String, group_id: String, topic: String) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("group.id", &group_id)
        .create()
        .expect("Consumer creation failed");

    consumer.subscribe(&[&topic]).expect("Subscribing failed");

    let mut stream = consumer.start();

    while let Some(message) = stream.next().await {
        // it might happen that the broker is not ready, so ignore those failures. There could
        // be other failures so in real-world code we should check the errors.
        match message {
            Ok(message) => {
                let payload = message.payload().unwrap();
                let greet_response: GreetResponse = protobuf::parse_from_bytes(payload).unwrap();
                println!(
                    "{}: {}",
                    greet_response.get_name(),
                    greet_response.get_greeting()
                );
            }
            Err(error) => log::error!("Error reading message: {}", error),
        }
    }
}
