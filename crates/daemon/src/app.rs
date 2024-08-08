mod connection;

pub use connection::{handle_server_connection, handle_socket_connection};

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use sail_core::socket::{SocketRequest, SocketResponse};
use std::convert::Infallible;

fn handle_socket_request(request: SocketRequest) -> SocketResponse {
    match request {
        SocketRequest::Greeting => SocketResponse::Okay,
    }
}

async fn handle_server_request(
    request: hyper::Request<Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
    todo!()
}
