use axum::response::{IntoResponse, Response};
use serde_json::json;

// TODO use thiserror
// TODO classify errors?
#[derive(Debug)]
pub enum Error {
    WebhookConfigError(String),
    WebhookNotFoundForPath(String),
    WebhookPayloadError(String),
    RulesNotFoundForWebhook(String),
    ToImplementError(String),
    ActionError(String),
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
            Error::WebhookPayloadError(message) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("webhook payload error: {message}"),
            ),
            _ => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
        };

        let body = axum::Json(json!({"error": error_message}));
        (status, body).into_response()
    }
}
