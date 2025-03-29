use crate::config::{loader, ServerConfig, Webhook};
use crate::server::{handlers, AppState, RuleMatcher}; // Added RuleMatcher import
use anyhow::Result; // Import Result
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap; // Added HashMap import
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Create the router with all routes
pub fn create_router(config: &ServerConfig) -> Result<Router<AppState>> { // Return Result
    // Load rule configurations
    let rule_configs =
        loader::load_rule_configs(config.clone().spec.rule_configs).unwrap_or_else(|err| {
            // TODO: Consider propagating this error as well if loading *any* config fails should stop server startup
            tracing::error!("Failed to load rule configs: {}", err);
            Vec::new()
        });

    // Create rule matcher
    let rule_matcher = Arc::new(RuleMatcher::new(rule_configs.clone()));
    let app_state = AppState { rule_matcher };

    // Create webhook routes
    let mut webhook_routes = Router::new();

    // Extract webhook configurations
    let webhooks = rule_configs
        .iter()
        .flat_map(|rule_config| rule_config.spec.webhooks.clone())
        .collect::<Vec<Webhook>>();

    // Add routes for each configured webhook
    for webhook in &webhooks {
        // Use ? to propagate the error
        webhook_routes = register_webhook_route(webhook_routes, webhook)?;
    }

    // Create main router
    let router = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::metrics))
        .nest("/webhook", webhook_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    Ok(router) // Return Ok(router)
}

/// Register webhook routes based on webhook type
fn register_webhook_route(
    webhook_routes: Router<AppState>,
    webhook: &Webhook,
) -> Result<Router<AppState>> { // Return Result
    let path = &webhook.path;
    let webhook_type = &webhook.webhook_type;

    match webhook_type.as_str() {
        "bitbucket" => {
            tracing::info!("Registering Bitbucket webhook: {}", path);
            Ok(webhook_routes.route(path, post(handlers::bitbucket::bitbucket_webhook))) // Updated path
        }
        // "github" => {
        //     tracing::info!("Registering GitHub webhook: {}", path);
        //     Ok(webhook_routes.route(path, post(handlers::webhook::github_webhook)))
        // }
        // "gitlab" => {
        //     tracing::info!("Registering GitLab webhook: {}", path);
        //     Ok(webhook_routes.route(path, post(handlers::webhook::gitlab_webhook)))
        // }
        _ => {
            let error_msg = format!("Unimplemented webhook type [{}] for path {}", webhook_type, path);
            // No need to trace error here, it will be handled by the caller
            Err(anyhow::anyhow!(error_msg)) // Return error directly
        }
    }
}
