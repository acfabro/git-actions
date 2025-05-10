use crate::app::{handlers, webhooks, AppState};
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Create a router with all routes for the webhooks
pub fn create_router() -> Router<Arc<AppState>> {
    // Create main router
    Router::new()
        // health check endpoint
        .route("/health", get(handlers::health_check))
        // metrics endpoint
        .route("/metrics", get(handlers::metrics))
        // webhook endpoint
        .route("/webhook/{*path}", post(webhooks::handler))
        // add tracing layer for logging
        .layer(TraceLayer::new_for_http())
}
