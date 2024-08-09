mod app;
mod configuration;
mod server;
mod socket;

use app::{handle_server_request, handle_socket_connection};
use configuration::Configuration;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown};
use server::Server;
use socket::Socket;
use std::{process::ExitCode, sync::Arc};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
};
use tracing::{error, info, span, Instrument, Level};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::TRACE)
        .init();

    let (stop_tx, mut stop_rx) = watch::channel(());

    tokio::spawn(async move {
        let mut termination = signal(SignalKind::terminate()).unwrap();
        termination.recv().await;
        stop_tx.send(()).unwrap();
    });

    let mut stop_rx_inner = stop_rx.clone();
    let socket_task_handle = tokio::spawn(
        async move {
            let socket = match Socket::attach() {
                Ok(socket) => socket,
                Err(error) => {
                    error!("failed to connect to socket: {error:?}");
                    return Err(error);
                }
            };

            info!("attached to systemd socket");

            loop {
                tokio::select! {
                    _ = stop_rx_inner.changed() => {
                        info!("received termination signal, stopping the socket handler!");
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

                        tokio::spawn(
                            async move {
                                handle_socket_connection(connection).await;
                            }.instrument(span!(Level::INFO, "handler")
                        ));
                    }
                }
            }

            Ok(())
        }
        .instrument(span!(Level::INFO, "socket")),
    );

    let configuration = Arc::new(match Configuration::from_filesystem() {
        Ok(configuration) => configuration,
        Err(error) => {
            error!("Failed to get configuration from filesystem: {error:?}");
            return ExitCode::FAILURE;
        }
    });

    let server = match Server::new(configuration).await {
        Ok(server) => server,
        Err(error) => {
            error!("failed to set up server: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let http_stack = http1::Builder::new();
    let shutdown_helper = GracefulShutdown::new();

    loop {
        tokio::select! {
            _ = stop_rx.changed() => {
                info!("received termination signal, stopping the server!");
                break
            },

            result = server.accept() => {
                let connection = match result {
                    Ok(connection) => connection,
                    Err(error) => {
                        error!("failed to accept connection: {error:?}");
                        continue
                    }
                };

                let io = TokioIo::new(connection.stream);
                let connection = http_stack.serve_connection(io, service_fn(handle_server_request));
                let future = shutdown_helper.watch(connection);

                tokio::spawn(async move {
                    if let Err(error) = future.await {
                        error!("failed to serve connection: {error:?}")
                    }
                })
            }
        };
    }

    if let Err(error) = socket_task_handle.await {
        error!("failed to complete socket handler task: {error:?}")
    };

    tokio::select! {
        _ = shutdown_helper.shutdown() => {
            info!("all connections gracefully closed");
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            error!("timed out wait for all connections to close");
        }
    }

    ExitCode::SUCCESS
}
