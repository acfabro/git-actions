use crate::app::webhooks::bitbucket::Bitbucket;
use crate::app::webhooks::types::{WebhookTypeHandler, Event};
use crate::app::{AppState, Error};
use crate::config::rules_config::{Action, HttpAction};
use crate::config::{Rule, WebhookSpec};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};
use Error::{RulesNotFoundForWebhook, WebhookNotFoundForPath};
use crate::app::webhooks::template_engine::render_template;
use tera::Context;

#[axum::debug_handler]
pub async fn handler(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, Error> {
    debug!("Handler path: {}", path);
    debug!(
        "Handler payload: {}",
        serde_json::to_string(&payload)
            .map_err(|_| Error::WebhookPayloadError("JSON Payload error".to_string()))?
    );

    // Get the webhook config based on the path
    let (webhook_name, webhook_config) = find_webhook(&state.config.webhooks, &path)?;
    debug!("Handler webhook: {}", webhook_name);

    // find rules subscribed to this webhook
    let webhook_rules = find_rules_for_webhook(&webhook_name, &state.config.rules)?;
    for (rule_name, rule) in webhook_rules.iter() {
        debug!(
            "Handler rule: {}, Description: {:?}",
            rule_name, rule.description
        );
    }

    // TODO - use a factory to create the handler based on the webhook type
    // Instantiate the webhook handler
    let handler = Bitbucket {
        config: webhook_config.bitbucket.unwrap_or_else(|| {
            // TODO implement config validation
            panic!("Bitbucket config is missing for webhook: {}", webhook_name)
        }),
        rules: webhook_rules,
        payload,
    };
    let actions = handler.run().await?;
    debug!("Handler actions: {:?}", actions);

    // Build the event struct for template context
    // (Assume Bitbucket handler exposes a method to extract Event)
    let event = handler.extract_event().await?;

    // exec the actions
    exec_actions(actions, &event).await?;

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

// TODO refactor to separate module?
async fn exec_actions(actions: Vec<&Action>, event: &Event) -> Result<(), Error> {
    for action in actions {
        if let Some(http) = &action.http {
            // TODO tracing
            exec_http_action(http, event).await?;
        }
        if let Some(_shell) = &action.shell {
            // TODO implement shell action
            error!("Shell action is not implemented yet");
        }
    }

    // TODO return some details about the action
    Ok(())
}

// TODO refactor to separate module?
async fn exec_http_action(action: &HttpAction, event: &Event) -> Result<(), Error> {
    // create a new HTTP client
    let client = reqwest::Client::new();

    // set request method
    let mut client = match action.method.to_uppercase().as_str() {
        "GET" => client.get(&action.url),
        "POST" => client.post(&action.url),
        "PUT" => client.put(&action.url),
        "PATCH" => client.patch(&action.url),
        "DELETE" => client.delete(&action.url),
        _ => {
            return Err(Error::ActionError(format!(
                "Unsupported HTTP method: {}",
                action.method
            )))
        }
    };

    // headers
    if let Some(headers) = &action.headers {
        for (key, value) in headers.iter() {
            client = client.header(key, value);
        }
    }

    // body (with template rendering)
    if let Some(body) = &action.body {
        let mut context = Context::new();
        context.insert("event", event);
        let rendered_body = match render_template(body, &context) {
            Ok(rendered) => rendered,
            Err(e) => {
                // Provide a clear error message with template and context info
                let context_keys = context
                    .as_json()
                    .as_object()
                    .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                return Err(Error::ActionError(format!(
                    "Template rendering error: {}\nTemplate: {}\nContext keys: {:?}",
                    e, body, context_keys
                )));
            }
        };
        client = client.body(rendered_body);
    }

    // send the request
    let response = client
        .send()
        .await
        .map_err(|e| Error::ActionError(e.to_string()))?;

    debug!("Action status: {}", response.status());

    Ok(())
}
