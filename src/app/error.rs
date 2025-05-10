use axum::response::{IntoResponse, Response};
use serde_json::json;

// TODO use thiserror
// TODO classify errors?
#[derive(Debug)]
pub enum Error {
    /// Invalid webhook configuration
    WebhookConfig(String),
    /// Webhook config not found given a path
    WebhookNotFoundForPath(String),
    /// No rule found attached to the webhook
    RulesNotFoundForWebhook(String),
    /// Error in the webhook handler
    Handler(String),
    /// Error performing the action
    Action(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::WebhookNotFoundForPath(path) => (
                axum::http::StatusCode::NOT_FOUND,
                format!("webhook not found for path: {path}"),
            ),
            Error::RulesNotFoundForWebhook(webhook_name) => (
                axum::http::StatusCode::NOT_FOUND,
                format!("rules not found for webhook: {webhook_name}"),
            ),
            Error::Action(message) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("action error: {message}"),
            ),
            Error::WebhookConfig(message) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("webhook configuration error: {message}"),
            ),
            Error::Handler(message) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("handler error: {message}"),
            ),
        };

        let body = axum::Json(json!({"error": error_message}));
        (status, body).into_response()
    }
}
