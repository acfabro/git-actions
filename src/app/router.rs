use crate::app::handlers;
use crate::config::Config;
// use crate::webhook::handlers::bitbucket::BitbucketHandler;
// use crate::webhook::handlers::webhook_type::{ApiConfig, WebhookTypeHandlerBase};
// use crate::webhook::{handlers::webhook::webhook_handler, service::WebhookEntry, Service};
use anyhow::Result;
use axum::routing::get;
use axum::Router;

/// Create a router with all routes for the webhooks
pub async fn create_router(config: &Config) -> Result<Router> {
    // Create webhook entries
    // let mut path_to_entry = HashMap::new();

    // for (webhook_name, webhook_config) in &config.webhooks {
    //     // TODO implement
    // }

    // Create webhook service
    // let webhook_service = Arc::new(Service {
    //     webhooks: config.webhooks.to_owned(),
    // });

    // Create app state
    // let app_state = AppState {
    //     webhook_service: webhook_service
    // };

    // Create main router
    let mut app: Router = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::metrics));
        // .layer(TraceLayer::new_for_http())
        // .with_state(app_state);

    Ok(app)
}
