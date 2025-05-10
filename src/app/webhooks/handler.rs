use crate::app::{
    config::{
        rules::{Action, HttpAction, Rule},
        WebhookConfig,
    },
    template,
    webhooks::bitbucket::Bitbucket,
    webhooks::types::{Event, WebhookTypeHandler},
    AppState, Error,
    Error::Handler,
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
use tera::Context;
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
            rule.description.as_deref().unwrap_or("").to_string()
        );
    }

    // TODO - use a factory to create the handler based on the webhook type
    let handler = create_bitbucket_handler(payload, webhook_config.to_owned(), webhook_rules)?;

    // run the webhook handler
    let actions = handler.run().await.map_err(|e| Handler(e.to_string()))?;
    debug!("Handler actions: {:?}", actions);

    // Extract the event for template rendering
    let event = handler
        .extract_event()
        .await
        .map_err(|e| Handler(e.to_string()))?;

    // exec the actions with the event for template context
    exec_actions(actions, &event).await?;

    // Return a success response
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": format!("Webhook processed: {}", &webhook_config.metadata.name),
        })),
    ))
}

fn create_bitbucket_handler(
    payload: Value,
    webhook_config: WebhookConfig,
    webhook_rules: HashMap<String, &Rule>,
) -> Result<Bitbucket, Error> {
    let bitbucket_config = webhook_config.spec.bitbucket.as_ref().ok_or_else(|| {
        Error::WebhookConfig(format!(
            "Bitbucket config is missing for webhook: {}",
            webhook_config.metadata.name
        ))
    })?;

    // Instantiate the webhook handler
    let handler = Bitbucket {
        config: bitbucket_config.to_owned(),
        rules: webhook_rules,
        payload,
    };
    Ok(handler)
}

// TODO refactor to separate module?
async fn exec_actions(actions: Vec<&Action>, event: &Event) -> Result<(), Error> {
    // Build the template context once with all environment variables
    let context = template::build_template_context(event);

    println!("{:?}", context);

    for action in actions {
        if let Some(http) = &action.http {
            // TODO tracing
            exec_http_action(http, &context).await?;
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
async fn exec_http_action(action: &HttpAction, context: &Context) -> Result<(), Error> {
    // create a new HTTP client
    let client = reqwest::Client::new();

    // Render templates in method and url
    let method = template::render_template(&action.method, context)
        .map_err(|e| Error::Action(format!("Failed to render method template: {}", e)))?;

    let url = template::render_template(&action.url, context)
        .map_err(|e| Error::Action(format!("Failed to render URL template: {}", e)))?;

    // set request method
    let mut client = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        _ => {
            return Err(Error::Action(format!(
                "Unsupported HTTP method: {}",
                method
            )))
        }
    };

    // headers with template rendering
    if let Some(headers) = &action.headers {
        let rendered_headers = template::render_template_map(headers, context);
        for (key, value) in rendered_headers.iter() {
            client = client.header(key, value);
        }
    }

    // body with template rendering
    if let Some(body) = &action.body {
        let rendered_body = template::render_template(body, context)
            .map_err(|e| Error::Action(format!("Failed to render body template: {}", e)));

        client = client.body(rendered_body?);
    }

    // send the request
    let response = client
        .send()
        .await
        .map_err(|e| Error::Action(e.to_string()))?;

    debug!("Action status: {}", response.status());

    Ok(())
}
