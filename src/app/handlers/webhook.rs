use crate::app::{AppState, Error};
use crate::config::{WebhookSpec, Rule};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use Error::{RulesNotFoundForWebhook, WebhookNotFoundForPath};

#[axum::debug_handler]
pub async fn webhook(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Error> {
    debug!("webhook: path={}, payload={:?}", path, payload);

    // 1. Get the webhook config based on the path
    let (webhook_name, _) = find_webhook(&state.config.webhooks, &path)?;

    info!("Processing webhook: {}", webhook_name);

    // 2. Check if the payload contains the event type
    let webhook_rules = find_rules_for_webhook(&webhook_name, &state.config.rules)?;

    // 3. Check if the payload contains the event type
    for (rule_name, rule) in webhook_rules.iter() {
        debug!("Rule: {}, Description: {:?}", rule_name, rule.description);
        debug!("Event types: {:?}", rule.event_types);
        debug!("Actions: {}", rule.actions.len());
    }

    // Return a success response
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": format!("Webhook processed: {webhook_name}"),
        })),
    )
        .into_response())
}

fn find_webhook(
    webhooks: &HashMap<String, WebhookSpec>,
    path: &str,
) -> Result<(String, WebhookSpec), Error> {
    // TODO - Implement a more efficient search for webhooks
    for (name, config) in webhooks.iter() {
        if config.path == path {
            return Ok((name.to_owned(), config.to_owned()));
        }
    }

    Err(WebhookNotFoundForPath(path.to_owned()).into())
}

fn find_rules_for_webhook<'a>(
    webhook_name: &'a String,
    rules: &'a HashMap<String, Rule>,
) -> Result<HashMap<String, &'a Rule>, Error> {
    let mut applicable_rules = HashMap::new();

    for (rule_name, rule) in rules.iter() {
        if rule.webhooks.contains(webhook_name) {
            applicable_rules.insert(rule_name.to_owned(), rule);
        }
    }

    if applicable_rules.is_empty() {
        return Err(RulesNotFoundForWebhook(webhook_name.to_owned()).into());
    }

    Ok(applicable_rules)
}
