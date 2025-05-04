use crate::app::{
    config::{
        rules::{Action, HttpAction, Rule},
        WebhookConfig,
    },
    webhooks::bitbucket::Bitbucket,
    webhooks::types::WebhookTypeHandler,
    AppState,
    Error,
    Error::HandlerError,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};
use Error::{RulesNotFoundForWebhook, WebhookNotFoundForPath};

#[axum::debug_handler]
pub async fn handler(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    debug!("Handler path={}", path);
    debug!("Handler payload={}", payload.to_string());

    // Get the webhook config based on the path
    let webhook_config = state
        .config
        .find_webhook_by_path(&path)
        .map_err(|_| WebhookNotFoundForPath(path.to_string()))?;
    debug!("Handler webhook: {}", webhook_config.metadata.name);

    // find rules subscribed to this webhook
    let webhook_rules = state
        .config
        .find_rules_by_webhook(&webhook_config.metadata.name)
        .map_err(|_| RulesNotFoundForWebhook(webhook_config.metadata.name.to_string()))?;
    for (rule_name, rule) in webhook_rules.iter() {
        debug!(
            "Handler rule: {}, Description: {}",
            rule_name,
            rule.description.to_owned().unwrap_or("".to_string())
        );
    }

    // TODO - use a factory to create the handler based on the webhook type
    let handler = create_bitbucket_handler(payload, webhook_config.to_owned(), webhook_rules)?;

    // run the webhook handler
    let actions = handler
        .run()
        .await
        .map_err(|e| HandlerError(e.to_string()))?;
    debug!("Handler actions: {:?}", actions);

    // exec the actions
    exec_actions(actions).await?;

    // Return a success response
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": format!("Webhook processed: {}", &webhook_config.metadata.name),
        })),
    )
        .into_response())
}

fn create_bitbucket_handler(
    payload: Value,
    webhook_config: WebhookConfig,
    webhook_rules: HashMap<String, &Rule>,
) -> Result<Bitbucket, Error> {
    let bitbucket_config = webhook_config.spec.bitbucket.as_ref().ok_or_else(|| {
        Error::WebhookConfigError(format!(
            "Bitbucket config is missing for webhook: {}",
            webhook_config.metadata.name
        ))
    })?;

    // Instantiate the webhook handler
    let handler = Bitbucket {
        config: bitbucket_config.to_owned(),
        rules: webhook_rules.to_owned(),
        payload,
    };
    Ok(handler)
}

// TODO refactor to separate module?
async fn exec_actions(actions: Vec<&Action>) -> Result<(), Error> {
    for action in actions {
        if let Some(http) = &action.http {
            // TODO tracing
            exec_http_action(http).await?;
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
async fn exec_http_action(action: &HttpAction) -> Result<(), Error> {
    // create a new HTTP client
    let client = reqwest::Client::new();

    // set request method
    let mut client = match action.method.to_uppercase().as_str() {
        "GET" => client.get(&action.url),
        "POST" => client.post(&action.url),
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

    // body
    if let Some(body) = &action.body {
        client = client.body(body.to_owned());
    }

    // send the request
    let response = client
        .send()
        .await
        .map_err(|e| Error::ActionError(e.to_string()))?;

    debug!("Action status: {}", response.status());

    Ok(())
}
