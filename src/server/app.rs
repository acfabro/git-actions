use anyhow::{Context, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::server::router;
use axum::{routing::get, Router};
use tokio::signal;
use tracing::info;

/// HTTP server for Git-Actions
pub struct App {
    config: Config,
}

impl App {
    /// Create a new HTTP server with the given configuration
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.to_owned(),
        }
    }

    /// Start the HTTP server
    pub async fn start(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.server_config.spec.host, self.config.server_config.spec.port);
        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Could not bind to {}", addr))?;

        info!("Server listening on {}", addr);

        // create a router
        let app = router::create_router(&self.config).await?;

        axum::serve(listener, app)
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .with_context(|| "Failed to start server")?;

        Ok(())
    }

    /// Shutdown the server
    async fn shutdown_signal() {
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
