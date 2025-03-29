use anyhow::{Context, Result};
use tokio::net::TcpListener;

use super::AppState;
use crate::config::ServerConfig;
use crate::server::router;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

/// HTTP server for Git-Actions
pub struct Server {
    config: ServerConfig,
}

impl Server {
    /// Create a new HTTP server with the given configuration
    pub fn new(config: &ServerConfig) -> Self {
        Self {
            config: config.to_owned(),
        }
    }

    /// Run the HTTP server
    pub async fn run(&mut self) -> Result<()> {
        // Create router (which includes state creation)
        // Use ? to handle potential errors from create_router
        let router = router::create_router(&self.config)?;

        // Build the server
        let addr = format!("{}:{}", self.config.spec.host, self.config.spec.port);
        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Could not bind to {}", addr))?;

        info!("Server listening on {}", addr);

        // Run the server with graceful shutdown
        axum::serve(listener, router)
            .with_graceful_shutdown(Self::shutdown())
            .await
            .with_context(|| "Server error")?;

        Ok(())
    }

    /// Shutdown the server
    async fn shutdown() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install ctrl+c handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => { info!("Got ctrl-c"); },
            _ = terminate => { info!("Got terminate signal"); },
        }

        info!("Shutting down");
    }
}
