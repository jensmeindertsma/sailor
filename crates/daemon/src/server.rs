use crate::configuration::Configuration;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    configuration: Arc<Configuration>,
    listener: TcpListener,
}

impl Server {
    pub async fn new(configuration: Arc<Configuration>) -> Result<Self, ServerError> {
        let listener = TcpListener::bind("127.0.0.1:4250")
            .await
            .map_err(ServerError::Bind)?;

        Ok(Self {
            configuration,
            listener,
        })
    }

    pub async fn accept(&self) -> Result<ServerConnection, ServerError> {
        let (stream, address) = self.listener.accept().await.map_err(ServerError::Accept)?;

        Ok(ServerConnection { stream, address })
    }
}

pub struct ServerConnection {
    pub stream: TcpStream,
    pub address: SocketAddr,
}

#[derive(Debug)]
pub enum ServerError {
    Accept(io::Error),
    Bind(io::Error),
}
