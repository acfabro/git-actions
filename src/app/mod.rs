pub mod config;
mod error;
mod handlers;
mod logging;
mod router;
mod server;
pub mod template;
pub mod webhooks;

pub use config::{Config, ServerConfig};
pub use error::Error;

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
    let mut app_config = Config::new();
    app_config.load(&server_config.spec.configs)?;

    // Create HTTP server
    Server::new(server_config, app_config).start().await?;

    Ok(())
}
