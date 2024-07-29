mod configuration;
mod interface;
mod server;

use configuration::Configuration;
use interface::Interface;
use server::Server;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    info!("starting");

    let mut tasks = JoinSet::new();

    let configuration: Arc<Configuration> = Arc::new(Configuration::from_filesystem().await);

    let config = configuration.clone();
    tasks.spawn(async move {
        // The interface attaches to the systemd socket to listen for and process request messages sent by the CLI tool `sail`.
        Interface::attach_to_systemd_socket(config)
            .handle_requests()
            .await
    });

    tasks.spawn(async move {
        let server = Server::with_config(configuration);

        server.start().await
    });

    while tasks.join_next().await.is_some() {}
}
