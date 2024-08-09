mod connection;

pub use connection::handle_socket_connection;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use sail_core::socket::{SocketRequest, SocketResponse};
use std::convert::Infallible;

fn handle_socket_request(request: SocketRequest) -> SocketResponse {
    match request {
        SocketRequest::Greeting => SocketResponse::Okay,
    }
}

pub async fn handle_server_request(
    request: hyper::Request<Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
    Ok(hyper::Response::new(Full::new(Bytes::from(format!(
        "Hello, World! '{}'",
        request.uri()
    )))))
}
