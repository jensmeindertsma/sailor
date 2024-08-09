mod app;
mod configuration;
mod server;
mod socket;

use app::{ServerHandler, SocketHandler};
use configuration::Configuration;
use hyper::server::conn::http1;
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown, service::TowerToHyperService};
use server::Server;
use socket::Socket;
use std::{process::ExitCode, sync::Arc, time::Duration};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::watch,
    time::sleep,
};
use tracing::{error, info, span, Instrument, Level};

// This is the entrypoint for the Sail application deployment daemon.
// Let's go over how it works:
// -> the daemon consists of two listener that work concurrently:
//  1. a Unix Socket listener, which attaches to the systemd provided `sail.socket`.
//     -) you can view the service and socket files under the `install` directory.
//     -) this socket listens to messages containing requests issued by the `sail`
//        CLI program located in `crates/CLI`. It processes these requests and replies
//        with a response that the CLI can read.
// 2. a TCP listener which, by default, binds to `127.0.0.1:4250`.
//     -) this listener processor incoming HTTP requests, and forwards them to
//        their destination, which can be one of three locations:
//          - a Sail-managed application container (based on the `Host` header)
//          - the Sail container registry, which is an Axum server implementing
//            an API that application CI infrastructure can push new images to
//            which are then automatically deployed.
//          - an error page server.
//          - (maybe in the future a web interface for Sail?)
//     -) this HTTP server/proxy implements graceful shutdown, finishing existing connection
//        before shutting down when it receives a SIGTERM signal.
//     -) the functionality of this proxy server can be controlled through persistent
//        configuration modified using the `sail` CLI.
//
// It is recommended you place the Sail daemon behind a full-featured proxy such as
// Nginx, to handle TLS and protect against many kinds of attacks. The Sail server is
// quite trusting of users' good intentions, and doesn't do any sophisticated defense,
// but the web is full of bad guys, so please make sure you put either Nginx or a
// external CDN server in front of this daemon.

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::TRACE)
        .init();

    // We set up a channel that we can poll to check whether the daemon
    // process has received a SIGTERM signal, for example when being stopped
    // with `systemctl stop sail`. This way we can initiate a graceful shutdown,
    // which means we stop accepting new connection and finish handling connections
    // that are already established.
    let (stop_tx, mut stop_rx) = watch::channel(());
    tokio::spawn(async move {
        let mut termination = signal(SignalKind::terminate()).unwrap();
        termination.recv().await;
        stop_tx.send(()).unwrap();
    });

    let configuration = Arc::new(match Configuration::from_filesystem() {
        Ok(configuration) => configuration,
        Err(error) => {
            error!("Failed to get configuration from filesystem: {error:?}");
            return ExitCode::FAILURE;
        }
    });

    let mut stop_rx_socket = stop_rx.clone();
    let configuration_socket = configuration.clone();
    // We place the socket handling logic into a separate
    // tokio task, so it can process requests without blocking the server.
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

            // Here we use a `tokio::select` macro to check whether the SIGTERM
            // signal has arrived randomly between serving new connections. Once
            // we receive the signal we break out of the loop, thus stopping the
            // acceptance of new connections.
            loop {
                tokio::select! {
                    _ = stop_rx_socket.changed() => {
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

                        let service = SocketHandler::new(configuration_socket.clone());
                        let future = socket::serve_connection(connection, service);

                        // Already established connections are spawned into their
                        // own task to help create non-blocking service and to allow
                        // existing connections to be finished before terminating.
                        // This currently does not implement any kind of graceful shutdown,
                        // but since requests are so short-lived and small we should be able
                        // to expect that this won't be a big problem. If a command fails due
                        // to the server quitting in the middle of it, the CLI user can just
                        // re-issue the command, this is less important than handling HTTP
                        // server connections gracefully. However, we might revisit this in the
                        // future since it is probably not too difficult to implement this manually.
                        // We'd need some kind of way to reply to all open messages with a
                        // `SocketResponse::Shutdown`.
                        tokio::spawn(async move {
                            if let Err(error) = future.await {
                                error!("failed to serve socket connection: {error:?}")
                            }.instrument(span!(Level::INFO, "handler"))
                        });
                    }
                }
            }

            Ok(())
        }
        .instrument(span!(Level::INFO, "socket")),
    );

    let server = match Server::new(configuration.clone()).await {
        Ok(server) => server,
        Err(error) => {
            error!("failed to set up server: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let http_stack = http1::Builder::new();
    let shutdown_helper = GracefulShutdown::new();

    let span = span!(Level::INFO, "server").entered();

    // Here, we again check for a shutdown signal between accepting new
    // incoming connections. We employ the `hyper-util` graceful shutdown utility
    // to handle existing connections when terminating the server. Read more about
    // what actually happens at https://github.com/hyperium/hyper/discussions/2515.
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

                let handler = ServerHandler::new(configuration.clone());

                // These three lines wrap the connection stream in a future-driven
                // data structure that handles the whole HTTP1.1 protocol, as well
                // as registering the connection for graceful shutdown.
                let io = TokioIo::new(connection.stream);
                let connection = http_stack.serve_connection(io, TowerToHyperService::new(handler));
                let future = shutdown_helper.watch(connection);

                // Each future is then moved into its own task, allowing the accept
                // loop to continue to the next connection in a non-blocking fashion.
                // The connection future is then driven to completion by the Tokio
                // executor.
                tokio::spawn(async move {
                    if let Err(error) = future.await {
                        error!("failed to serve http connection: {error:?}")
                    }.instrument(span!(Level::INFO, "handler"))
                })
            }
        };
    }

    tokio::select! {
        _ = shutdown_helper.shutdown() => {
            info!("all connections gracefully closed");
        },
        _ = sleep(Duration::from_secs(10)) => {
            error!("timed out wait for all connections to close");
        }
    }

    span.exit();

    let span = span!(Level::INFO, "daemon").entered();

    info!("succesfully stopped the server");

    if let Err(error) = socket_task_handle.await {
        error!("failed to complete socket handler task: {error:?}")
    } else {
        info!("successfully stopped the socket handler")
    }

    span.exit();

    ExitCode::SUCCESS
}
