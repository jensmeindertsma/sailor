use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SocketMessage {
    pub id: u8,
    pub request: SocketRequest,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SocketRequest {
    Greeting,
}

#[derive(Deserialize, Serialize)]
pub struct SocketReply {
    pub regarding: u8,
    pub response: SocketResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SocketResponse {
    Okay,
}
