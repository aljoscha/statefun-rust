//! `Transport` that uses [Hyper](http://docs.rs/hyper) to serve stateful functions.
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use bytes::buf::BufExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{http, Body, Request, Response, Server};
use protobuf::{Message, ProtobufError};
use thiserror::Error;
use tokio::runtime;

use statefun_proto::request_reply::ToFunction;

use crate::function_registry::FunctionRegistry;
use crate::invocation_bridge::InvocationBridge;
use crate::transport::hyper::HyperTransportError::TokioInitializationFailure;
use crate::transport::Transport;
use crate::InvocationError;

/// A [Transport](crate::transport::Transport) that serves stateful functions on a http endpoint at
/// the given `bind_address`.
pub struct HyperHttpTransport {
    bind_address: SocketAddr,
}

impl HyperHttpTransport {
    /// Creates a new `HyperHttpTransport` that can serve stateful functions at the given
    /// `bind_address`.
    pub fn new(bind_address: SocketAddr) -> HyperHttpTransport {
        HyperHttpTransport { bind_address }
    }
}

impl Transport for HyperHttpTransport {
    type Error = HyperTransportError;

    fn run(self, function_registry: FunctionRegistry) -> Result<(), Self::Error> {
        log::info!(
            "Hyper transport will start listening on {}",
            self.bind_address
        );

        let runtime = runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build();
        let mut runtime = match runtime {
            Ok(rt) => rt,
            Err(error) => return Err(TokioInitializationFailure(error)),
        };

        let function_registry = Arc::new(Mutex::new(function_registry));

        runtime.block_on(async {
            let make_svc = make_service_fn(|_conn| {
                let function_registry = Arc::clone(&function_registry);
                async move {
                    Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                        let function_registry = Arc::clone(&function_registry);
                        async move { handle_request(function_registry, req).await }
                    }))
                }
            });
            let server = Server::bind(&self.bind_address).serve(make_svc);
            let graceful = server.with_graceful_shutdown(shutdown_signal());

            if let Err(e) = graceful.await {
                eprintln!("server error: {}", e);
            }
        });

        Ok(())
    }
}

async fn handle_request(
    function_registry: Arc<Mutex<FunctionRegistry>>,
    req: Request<Body>,
) -> Result<Response<Body>, HyperTransportError> {
    let (_parts, body) = req.into_parts();
    log::debug!("Parts {:#?}", _parts);

    let full_body = hyper::body::to_bytes(body).await?;
    log::debug!("--drey: full body: {:?}", full_body);
    let to_function: ToFunction = protobuf::parse_from_reader(&mut full_body.reader())?;
    let from_function = {
        let function_registry = function_registry.lock().unwrap();
        function_registry.invoke_from_proto(to_function)?
    };

    log::debug!("Response: {:#?}", from_function);

    let encoded_result = from_function.write_to_bytes()?;

    let response = Response::builder()
        .header("content-type", "application/octet-stream")
        .body(encoded_result.into())?;

    log::debug!("Succesfully encoded response.");

    Ok(response)
}

/// The error type for the `HyperHttpTransport` `Transport`.
///
/// Errors can originate from many different source because a `Transport` is the entry point that
/// pulls everything together. This mostly wraps error types of other crates/modules that we use.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum HyperTransportError {
    /// Something went wrong with Protobuf parsing, writing, packing, or unpacking.
    #[error(transparent)]
    ProtobufError(#[from] ProtobufError),

    /// An error occurred while invoking a user function.
    #[error(transparent)]
    InvocationError(#[from] InvocationError),

    /// An error from the underlying hyper
    #[error(transparent)]
    HyperError(#[from] hyper::error::Error),

    /// An error from the underlying hyper/http.
    #[error(transparent)]
    HttpError(#[from] http::Error),

    /// Something went wrong with Tokio.
    #[error("Tokio runtime could not be initialized")]
    TokioInitializationFailure(#[source] std::io::Error),
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
