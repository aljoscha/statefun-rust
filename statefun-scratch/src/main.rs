use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::buf::BufExt;
use exitfailure::ExitFailure;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use protobuf::Message;

use statefun_protos::http_function::FromFunction;
use statefun_protos::http_function::ToFunction;
use statefun_protos::http_function::ToFunction_oneof_request;
use statefun_scratch_protos::example::GreetRequest;

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, failure::Error> {
    let (_parts, body) = req.into_parts();
    log::info!("Parts {:#?}", _parts);

    let full_body = hyper::body::to_bytes(body).await?;
    let to_function: ToFunction = protobuf::parse_from_reader(&mut full_body.reader())?;

    let batch_request = to_function.request.expect("No request was sent.");
    let ToFunction_oneof_request::invocation(batch_request) = batch_request;

    log::info!("Got batch request {:#?}", batch_request);

    for invocation in batch_request.invocations.into_iter() {
        let argument = invocation.argument.unwrap();
        let _greet_request: GreetRequest = argument.unpack()?.unwrap();
        log::info!("We should greet {:#?}", _greet_request);
    }

    let from_function = FromFunction::new();

    let encoded_result = from_function.write_to_bytes()?;

    let response = Response::builder()
        .header("content-type", "application/octet-stream")
        .body(encoded_result.into())?;

    Ok(response)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    env_logger::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(hello_world)) });

    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
