use crate::config::Config;
use crate::server::{handlers, AppState};
use crate::webhook::handlers::bitbucket::BitbucketHandler;
use crate::webhook::handlers::webhook_type::{ApiConfig, WebhookTypeHandlerBase};
use crate::webhook::{handlers::webhook::webhook_handler, service::WebhookEntry, WebhookService};
use anyhow::{Context, Result};
use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;
use axum::routing::{get, post};
// use tower_http::trace::TraceLayer;
use tracing::info;

/// Create a router with all routes for the webhooks
pub async fn create_router(config: &Config) -> Result<Router> {
    // Create webhook entries
    let mut path_to_entry = HashMap::new();

    for (webhook_name, webhook_config) in &config.webhooks {
        // TODO implement
    }

    // Create webhook service
    let webhook_service = Arc::new(WebhookService::new(
        path_to_entry,
        config.rules.clone(),
    ));

    // Create app state
    let app_state = AppState {
        webhook_service: webhook_service.clone(),
    };
    
    // Create main router
    let mut app: Router = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::metrics))
        // .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    
    // Register webhook routes
    for path in webhook_service.get_all_paths() {
        info!("Registering webhook route: {}", path);
        app = app.route(path, post(webhook_handler));
    }
    
    Ok(app)
}
