use super::{handle_server_request, handle_socket_request};
use crate::{server::ServerConnection, socket::SocketConnection};
use hyper::{server::conn::http1::Builder as ConnectionBuilder, service::service_fn};
use hyper_util::{
    rt::TokioIo,
    server::graceful::{GracefulConnection, GracefulShutdown},
};
use sail_core::socket::SocketReply;
use tokio::pin;
use tracing::{error, info};

pub async fn handle_socket_connection(mut connection: SocketConnection) {
    // Most of the time each connection only makes a single request, but we
    // spawn a task anyway to prevent blocking when multiple requests are sent
    // or the requests take a long time to process.
    info!("handling connection from address {:?}", connection.address);

    loop {
        let result = connection.accept().await;
        let message = match result {
            Ok(maybe_message) => match maybe_message {
                None => break,
                Some(message) => message,
            },
            Err(error) => {
                error!("failed to read incoming message: {error:?}");
                continue;
            }
        };

        info!(
            id = message.id,
            "received socket request = {:?}", message.request,
        );

        let response = handle_socket_request(message.request);

        info!(
            id = message.id,
            "replying to socket request = {:?}", response
        );

        if let Err(error) = connection
            .reply(SocketReply {
                regarding: message.id,
                response,
            })
            .await
        {
            error!("failed to send reply: {error:?}")
        };
    }
}

pub async fn handle_server_connection(
    connection: ServerConnection,
    http_stack: &ConnectionBuilder,
    shutdown_helper: &GracefulShutdown,
) -> Result<(), hyper::Error> {
    info!("serving connection from {}", connection.address);

    let io = TokioIo::new(connection.stream);
    let conn = http_stack.serve_connection(io, service_fn(handle_server_request));
    pin!(conn);
    shutdown_helper.watch(conn);

    conn.await
}
