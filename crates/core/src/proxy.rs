use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ProxyError {
    FetchError(FetchError),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FetchError {
    Connection(String),
    Handshake(String),
    Send(String),
}
