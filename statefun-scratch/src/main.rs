use std::convert::Infallible;
use std::net::SocketAddr;

use exitfailure::ExitFailure;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use prost::Message;

use bytes::BytesMut;
use statefun_protos::http_function::to_function;
use statefun_protos::http_function::FromFunction;
use statefun_protos::http_function::ToFunction;
use statefun_scratch_protos::example::GreetRequest;

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, failure::Error> {
    let (parts, body) = req.into_parts();
    log::info!("Parts {:#?}", parts);

    let full_body = hyper::body::to_bytes(body).await?;
    let to_function = ToFunction::decode(full_body)?;

    let batch_request = to_function.request.expect("No request was sent.");
    let to_function::Request::Invocation(batch_request) = batch_request;

    log::info!("Got batch request {:#?}", batch_request);

    for invocation in batch_request.invocations {
        let argument = invocation.argument.expect("No argument given.");
        let greet_request = GreetRequest::decode(argument.value.as_ref())?;
        log::info!("We should greet {:#?}", greet_request);
    }

    let from_function = FromFunction { response: None };

    let mut buffer = BytesMut::new();
    from_function.encode(&mut buffer)?;

    let response = Response::builder()
        .header("content-type", "application/octet-stream")
        .body(buffer.freeze().into())?;

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
