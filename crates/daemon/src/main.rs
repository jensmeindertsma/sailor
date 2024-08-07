#![feature(duration_constructors)]

mod socket;

use sail_core::{Reply, Request, Response};
use socket::{Socket, SocketConnection};
use std::{
    process::{ExitCode, Termination},
    time::Duration,
};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
    task::JoinSet,
    time::{sleep_until, Instant},
};
use tracing::{debug, error, info, instrument, span, Instrument, Level, Span};

#[tokio::main]
async fn main() -> impl Termination {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::TRACE)
        .init();

    let (stop_tx, mut stop_rx) = watch::channel(());

    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        sigterm.recv().await;
        stop_tx.send(()).unwrap();
    });

    let mut tasks = JoinSet::new();

    let socket_span = span!(Level::INFO, "socket");

    let stop_rx_inner = stop_rx.clone();
    let socket_span_inner = socket_span.clone();
    tasks.spawn(
        async move {
            let mut stop_rx = stop_rx_inner;
            let socket = match Socket::attach() {
                Ok(socket) => socket,
                Err(error) => {
                    error!("failed to connect to socket: {error:?}");
                    return Err(());
                }
            };

            info!("attached to systemd socket");

            loop {
                tokio::select! {
                    biased;

                    _ = stop_rx.changed() => {
                        info!("received SIGTERM signal, stopping the socket handler!");
                        break
                    },

                    result = socket.accept() => {
                        let connection = match result {
                            Ok(connection) => connection,
                            Err(error) => {
                                error!("failed to accept socket connection: {error:?}");
                                continue;
                            }
                        };

                        handle_socket_connection(connection, socket_span_inner.clone()).await;
                    }
                }
            }

            Ok(())
        }
        .instrument(socket_span),
    );

    tasks.spawn(
        async move {
            //let server = Server::new(configuration);

            tokio::select! {
                biased;

                _ = stop_rx.changed() => {
                    info!("received SIGTERM signal, stopping the server!");
                },

                // TODO: accept server connections here
                _ = sleep_until(Instant::now() + Duration::from_days(100)) => {}
            }

            Ok(())
        }
        .instrument(span!(Level::INFO, "server")),
    );

    let mut completed = 0;
    while let Some(result) = tasks.join_next().await {
        debug!("completed task {completed}, {} remaining", tasks.len());
        completed += 1;

        match result {
            Ok(result) => {
                if result.is_err() {
                    error!("task failed!")
                }
            }
            Err(error) => {
                error!("task failed to join: {error:?}")
            }
        }
    }

    ExitCode::SUCCESS
}

async fn handle_socket_connection(mut connection: SocketConnection, span: Span) {
    let handler_span = span!(Level::INFO, "handler");
    let handler_span_inner = handler_span.clone();
    let _ = async move {
        // Most of the time each connection only makes a single request, but we
        // spawn a task anyway to prevent blocking when multiple requests are sent
        // or the requests take a long time to process.
        info!(
            "handling connection from address {:?}",
            connection.socket_address
        );

        tokio::spawn(
            async move {
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
                        .reply(Reply {
                            regarding: message.id,
                            response,
                        })
                        .await
                    {
                        error!("failed to send reply: {error:?}")
                    };
                }
            }
            .instrument(handler_span_inner.clone()),
        )
        .await
    }
    .await;
}

fn handle_socket_request(request: Request) -> Response {
    match request {
        Request::Greeting => Response::Okay,
    }
}
