pub mod server;

pub use crate::Error;

mod handlers;
mod router;
mod logging;
mod webhooks;

#[cfg(test)]
mod tests;

use crate::config::{Config, ServerConfig};
use anyhow::{Context, Result};
use server::Server;

/// Application state shared across request handlers
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
}

/// Run the HTTP server with the given configuration
pub async fn run(server_config: ServerConfig) -> Result<()> {
    // Set up logging based on configuration
    logging::setup(&server_config.spec.logging).with_context(|| "Failed to setup logging")?;

    // Load application configuration
    let app_config = Config::load(&server_config.spec.configs)?;

    // Create HTTP server
    Server::new(server_config, app_config).start().await?;

    Ok(())
}
