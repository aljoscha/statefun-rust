use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use bytes::buf::BufExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use protobuf::Message;
use tokio::runtime;

use statefun_protos::http_function::ToFunction;

use crate::internal::FunctionRegistry;
use crate::transport::Transport;

pub struct HyperHttpTransport {
    bind_address: SocketAddr,
}

impl HyperHttpTransport {
    pub fn new(bind_address: SocketAddr) -> HyperHttpTransport {
        HyperHttpTransport { bind_address }
    }
}

impl Transport for HyperHttpTransport {
    fn run(self, function_registry: FunctionRegistry) -> Result<(), failure::Error> {
        log::info!(
            "Hyper transport will start listening on {}",
            self.bind_address
        );

        let mut runtime = runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()?;

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
) -> Result<Response<Body>, failure::Error> {
    let (_parts, body) = req.into_parts();
    log::debug!("Parts {:#?}", _parts);

    let full_body = hyper::body::to_bytes(body).await?;
    let to_function: ToFunction = protobuf::parse_from_reader(&mut full_body.reader())?;
    let from_function = {
        let function_registry = function_registry.lock().unwrap();
        function_registry.invoke(to_function)?
    };

    log::debug!("Response: {:#?}", from_function);

    let encoded_result = from_function.write_to_bytes()?;

    let response = Response::builder()
        .header("content-type", "application/octet-stream")
        .body(encoded_result.into())?;

    log::debug!("Succesfully encoded response.");

    Ok(response)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
