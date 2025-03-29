use axum::response::{IntoResponse, Response};
use serde_json::json;

// TODO use thiserror
#[derive(Debug)]
pub enum Error {
    WebhookNotFoundForPath(String),
    RulesNotFoundForWebhook(String),
    ToImplementError(String),
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
            _ => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
        };

        let body = axum::Json(json!({"error": error_message}));
        (status, body).into_response()
    }
}
