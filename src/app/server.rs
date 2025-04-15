use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::app::{router, AppState};
use crate::config::{Config, ServerConfig};
use tokio::signal;
use tracing::info;

/// HTTP server for Git-Actions
pub struct Server {
    server_config: ServerConfig,
    app_config: Config,
}

impl Server {
    /// Create a new HTTP server with the given configuration
    pub fn new(
        server_config: ServerConfig,
        app_config: Config,
    ) -> Self {
        Self { server_config, app_config }
    }

    /// Start the HTTP server
    pub async fn start(&mut self) -> Result<()> {
        let host = self.server_config.spec.host.clone();
        let port = self.server_config.spec.port;
        let addr = format!("{}:{}", host, port);

        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Could not bind to {}", addr))?;

        info!("Server listening on {}", addr);

        // create a router
        let app = router::create_router();

        // add app state
        let state = AppState {
            config: self.app_config.to_owned(),
        };
        let app = app.with_state(Arc::new(state));

        // start the server
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
