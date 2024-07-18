use super::application::Application;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Message {
    pub id: u16,
    pub request: Request,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Reply {
    pub regarding: u16,
    pub response: Response,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Request {
    CreateApplication { application: Application },
    DeleteApplication { hostname: String },
    GetApplications,
    Status,
    ValidateConfiguration,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Response {
    Error {
        message: String,
    },
    Success,
    Status {
        port: u16,
        applications: Vec<Application>,
    },
    Applications {
        applications: Vec<Application>,
    },
}
