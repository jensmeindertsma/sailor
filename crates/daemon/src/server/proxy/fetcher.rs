use std::net::SocketAddr;

use hyper::{body::Incoming, Request, Response};
use hyper_util::rt::TokioIo;
use sailor_core::proxy::FetchError;
use tokio::net::TcpStream;
use tracing::{error, info, instrument};

#[instrument(skip(request))]
pub async fn fetch(
    address: SocketAddr,
    request: Request<Incoming>,
) -> Result<Response<Incoming>, FetchError> {
    info!("fetching {address} uri: {}", request.uri());

    let stream = TcpStream::connect(address)
        .await
        .map_err(|e| FetchError::Connection(e.to_string()))?;

    info!("connected to stream");

    let io = TokioIo::new(stream);

    let (mut sender, connection) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| FetchError::Handshake(e.to_string()))?;

    info!("finished handshake");

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::spawn(async move {
        if let Err(err) = connection.await {
            error!("Connection failed: {:?}", err);
        }
    });

    let response = sender
        .send_request(request)
        .await
        .map_err(|e| FetchError::Send(e.to_string()));

    info!("got response: {:?}", response);

    response
}
