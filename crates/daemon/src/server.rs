mod proxy;

use super::configuration::Configuration;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as ConnectionBuilder,
    service::TowerToHyperService,
};
use proxy::Proxy;
use sailor_config::Configurable;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

pub struct Server {
    config: Arc<Configuration>,
}

impl Server {
    pub fn with_config(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    pub async fn start(&self) {
        let port = self.config.get().core.port;
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(address)
            .await
            .unwrap_or_else(|_| panic!("binding to localhost:{port} failed!"));

        while let Ok((stream, address)) = listener.accept().await {
            self.handle_connection(stream, address).await
        }
    }

    async fn handle_connection(&self, stream: TcpStream, client_address: SocketAddr) {
        info!("handling connection from {}", client_address);

        let configuration = self.config.clone();
        let proxy = TowerToHyperService::new(Proxy::new(configuration));
        let connection = ConnectionBuilder::new(TokioExecutor::new())
            .serve_connection(TokioIo::new(stream), proxy)
            .await;

        match connection {
            Ok(_) => info!("Finished serving connection to {}", client_address),
            Err(e) => error!(
                "error while serving connection to {}: {:?}",
                client_address, e
            ),
        }
    }
}
