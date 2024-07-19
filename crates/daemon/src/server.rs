mod proxy;

use super::configuration::Configuration;
use hyper::{
    rt::{Read, Write},
    server::conn::http1::Builder,
};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use proxy::Proxy;
use sailor_config::Configurable;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{error, info};

pub struct Server {
    config: Arc<Configuration>,
    connection_builder: Arc<Builder>,
}

impl Server {
    pub fn with_config(config: Arc<Configuration>) -> Self {
        Self {
            config,
            connection_builder: Arc::new(Builder::new()),
        }
    }

    pub async fn start(&self) {
        let port = self.config.get().core.port;
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(address)
            .await
            .unwrap_or_else(|_| panic!("binding to localhost:{port} failed!"));

        while let Ok((stream, address)) = listener.accept().await {
            self.handle_connection(TokioIo::new(stream), address).await
        }
    }

    async fn handle_connection<S>(&self, stream: S, client_address: SocketAddr)
    where
        S: Read + Write + Send + Unpin + 'static,
    {
        info!("handling connection from {}", client_address);

        let connection_builder = self.connection_builder.clone();
        let configuration = self.config.clone();
        let proxy = TowerToHyperService::new(Proxy::new(configuration));

        tokio::spawn(async move {
            let result = connection_builder.serve_connection(stream, proxy).await;

            match result {
                Ok(_) => info!("Finished serving connection to {}", client_address),
                Err(e) => error!(
                    "error while serving connection to {}: {:?}",
                    client_address, e
                ),
            }
        });
    }
}
