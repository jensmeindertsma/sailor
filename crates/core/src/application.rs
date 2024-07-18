use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Application {
    pub hostname: String,
    pub address: SocketAddr,
}
