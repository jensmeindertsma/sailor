use sail_core::application::Application;
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::Arc};

pub trait Configurable {
    fn get(&self) -> Arc<CurrentConfiguration>;
    fn set(&self, new: CurrentConfiguration) -> impl Future<Output = ()>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentConfiguration {
    pub core: CoreConfiguration,
    pub applications: Vec<Application>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoreConfiguration {
    pub port: u16,
}
