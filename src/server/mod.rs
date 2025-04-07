pub mod error;
pub mod handlers;
pub mod router;
pub mod app;

use crate::config::Config;
use crate::logging;
use crate::webhook::WebhookService;
use anyhow::{Context, Result};
use app::App;
use std::sync::Arc;
use tracing::info;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub webhook_service: Arc<WebhookService>,
}

/// Run the HTTP server with the given configuration
pub async fn run(config: Config) -> Result<()> {
    // Set up logging based on configuration
    logging::setup(&config.server_config.spec.logging)
        .with_context(|| "Failed to setup logging")?;

    // Create HTTP server
    App::new(&config).start().await?;

    Ok(())
}
