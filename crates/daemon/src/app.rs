mod connection;

pub use connection::handle_socket_connection;

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    Request as HyperRequest, Response as HyperResponse, StatusCode,
};
use sail_core::socket::{SocketRequest, SocketResponse};
use std::{convert::Infallible, sync::Arc};
use tower::Service;

use crate::configuration::Configuration;

fn handle_socket_request(request: SocketRequest) -> SocketResponse {
    match request {
        SocketRequest::Greeting => SocketResponse::Okay,
    }
}

pub struct Handler {
    configuration: Arc<Configuration>,
}
impl Handler {
    pub fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    // This is the main server function that proxies all requests to their destination.
    pub async fn handle_server_request(
        &self,
        request: HyperRequest<Incoming>,
    ) -> Result<HyperResponse<Full<Bytes>>, Infallible> {
        Ok(match request.headers().get("Host") {
            None => HyperResponse::builder()
                .header("Content-Type", "text/html")
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(format!(
                    "<h1>No Host header!</h1>\n"
                ))))
                .unwrap(),
            Some(host) => match host.to_str().unwrap() {
                "registry.jensmeindertsma.com" => HyperResponse::builder()
                    .header("Content-Type", "text/html")
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::from(format!(
                        "<h1>Registry {}</h1>\n",
                        request.uri()
                    ))))
                    .unwrap(),
                host => {
                    if let Some(application) = self.configuration.get().applications.get(host) {
                        HyperResponse::builder()
                            .header("Content-Type", "text/html")
                            .status(StatusCode::OK)
                            .body(Full::new(Bytes::from(format!(
                                "<h1>Application {} {}</h1>\n",
                                application.name,
                                request.uri()
                            ))))
                            .unwrap()
                    } else {
                        HyperResponse::builder()
                            .header("Content-Type", "text/html")
                            .status(StatusCode::BAD_REQUEST)
                            .body(Full::new(Bytes::from(format!(
                                "<h1>Unknown host {}!</h1>\n",
                                host
                            ))))
                            .unwrap()
                    }
                }
            },
        })
    }
}

impl Service<HyperRequest<Incoming>> for Handler {
    type Response = HyperResponse<HandlerBody>;
    type Future = HandlerFuture;
    type Error = Infallible;
}

struct HandlerFuture {}

impl Future for HandlerFuture {}

enum HandlerBody {}

impl hyper::body::Body for HandlerBody {}
