use crate::config::{loader, ServerConfig, Webhook};
use crate::server::{handlers, AppState, RuleMatcher};
use anyhow::{Context, Error, Result};
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::error;

/// Create the router with all routes
pub fn from_server_config(config: &ServerConfig) -> Result<Router> {
    // All rule configurations
    let rule_configs =
        loader::load_rule_configs(config.clone().spec.rule_configs.unwrap_or_default())
            .with_context(|| "Failed to load rule configurations")?;

    // Create rule matcher
    let rule_matcher = Arc::new(RuleMatcher::new(&rule_configs));
    let app_state = AppState { rule_matcher };

    // Extract webhook configurations
    let mut webhooks = Vec::new();
    for rule_config in &rule_configs {
        for (_, webhook) in &rule_config.spec.webhooks {
            webhooks.push(webhook.clone());
        }
    }

    // Create webhook routes
    let mut webhook_routes = Router::new();
    for webhook in &webhooks {
        // Use ? to propagate the error
        webhook_routes = register_webhook_route(webhook_routes, webhook)?;
    }

    // Create main router
    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::metrics))
        .nest("/webhooks", webhook_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    Ok(app)
}

/// Register webhook routes based on webhook type
fn register_webhook_route(
    webhook_routes: Router<AppState>,
    webhook: &Webhook,
) -> Result<Router<AppState>> {
    // Return Result
    let path = &webhook.path;
    let webhook_type = &webhook.webhook_type;

    match webhook_type.as_str() {
        "bitbucket" => {
            tracing::info!("Registering Bitbucket webhook: {}", path);
            Ok(webhook_routes.route(path, post(handlers::bitbucket::bitbucket_webhook)))
        }
        // throw errors for unimplemented webhook types
        _ => Err(Error::msg(format!(
            "Unimplemented webhook type [{}] for path: {}",
            webhook_type, path
        ))),
    }
}
