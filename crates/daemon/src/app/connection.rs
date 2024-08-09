use super::handle_socket_request;
use crate::socket::SocketConnection;

use sail_core::socket::SocketReply;
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
