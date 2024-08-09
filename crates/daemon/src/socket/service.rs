use super::SocketConnection;
use sail_core::socket::{SocketReply, SocketRequest, SocketResponse};
use std::{convert::Infallible, future::Future};
use tower::Service;
use tracing::{error, info};

pub async fn serve_connection<S, F>(
    mut connection: SocketConnection,
    mut service: S,
) -> Result<(), Infallible>
where
    S: Service<SocketRequest, Response = SocketResponse, Error = Infallible, Future = F>,
    F: Future<Output = Result<SocketResponse, Infallible>>,
{
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

        let response = service.call(message.request).await?;

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

    Ok(())
}
