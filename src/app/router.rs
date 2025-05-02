use crate::app::{handlers, webhooks, AppState};
use crate::config::ServerConfig;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Create a router with all routes for the webhooks
pub fn create_router(server_config: &ServerConfig) -> Router<Arc<AppState>> {
    // Create main router
    let app = Router::new()
        // health check endpoint
        .route("/health", get(handlers::health_check))
        // metrics endpoint
        .route("/metrics", get(handlers::metrics))
        // add route for webhooks
        .route(
            &format!("{}/{{*path}}", server_config.spec.context),
            post(webhooks::handler),
        )
        .layer(TraceLayer::new_for_http());

    app
}
