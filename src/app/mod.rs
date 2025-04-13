pub mod error;
pub mod handlers;
pub mod router;
pub mod server;

use crate::config::Config;
use crate::logging;
//use crate::webhook::Service;
use anyhow::{Context, Result};
use server::Server;
use std::sync::Arc;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    // pub webhook_service: Arc<Service>,
}

/// Run the HTTP server with the given configuration
pub async fn run(config: Config) -> Result<()> {
    // Set up logging based on configuration
    logging::setup(&config.server_config.spec.logging)
        .with_context(|| "Failed to setup logging")?;

    // Create HTTP server
    Server::new(&config).start().await?;

    Ok(())
}
