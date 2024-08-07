use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub id: u8,
    pub request: Request,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Request {
    Greeting,
}

#[derive(Deserialize, Serialize)]
pub struct Reply {
    pub regarding: u8,
    pub response: Response,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Okay,
}
