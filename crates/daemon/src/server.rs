mod proxy;

use super::configuration::Configuration;
use hyper::server::conn::http1::Builder as ConnectionBuilder;
use hyper_util::{rt::TokioIo, server::graceful::GracefulShutdown, service::TowerToHyperService};
use proxy::Proxy;
use sail_config::Configurable;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    select,
    signal::unix::{signal, SignalKind},
    sync::watch,
    time::sleep,
};
use tracing::{error, info};

pub struct Server {
    config: Arc<Configuration>,
    http: ConnectionBuilder,
}

impl Server {
    pub fn with_config(config: Arc<Configuration>) -> Self {
        Self {
            config,
            http: ConnectionBuilder::new(),
        }
    }

    pub async fn start(&self) {
        let port = self.config.get().core.port;
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(address)
            .await
            .unwrap_or_else(|_| panic!("binding to localhost:{port} failed!"));

        let (stop_tx, mut stop_rx) = watch::channel(());

        tokio::spawn(async move {
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            sigterm.recv().await;
            stop_tx.send(()).unwrap();
        });

        let graceful = GracefulShutdown::new();

        loop {
            select! {
                biased;

                _ = stop_rx.changed() => {
                    info!("received SIGTERM signal!");
                    break},
                Ok((stream, address)) = listener.accept() => {
                    let configuration = self.config.clone();
                    let http = self.http.clone();
                    let io = TokioIo::new(stream);

                    let proxy = TowerToHyperService::new(Proxy::new(configuration));

                    info!("serving connection from {address}");

                    let connection = http.serve_connection(io, proxy);
                    let future = graceful.watch(connection);

                    tokio::spawn(async move {
                        if let Err(error) = future.await {
                            error!("Error while handling connection: {error:?}")
                        }
                    })

                }
            };
        }

        tokio::select! {
            _ = graceful.shutdown() => {
                info!("all connections gracefully closed");
            },
            _ = sleep(Duration::from_secs(10)) => {
                error!("timed out wait for all connections to close");
            }
        }
    }
}
