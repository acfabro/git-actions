<![CDATA[use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Errors that can occur during webhook data extraction
#[derive(Debug, Error)]
pub enum WebhookDataError {
    #[error("Missing event type in payload")]
    MissingEventType,

    #[error("Missing branch information in payload")]
    MissingBranch,

    #[error("Failed to parse payload field: {0}")]
    ParseError(String),
}

/// Errors that can occur during webhook processing
#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Invalid webhook payload: {0}")]
    PayloadError(String), // From WebhookDataError or other payload issues

    #[error("Failed to execute action: {0}")]
    ActionExecutionError(String),

    #[error("Webhook type not implemented: {0}")]
    UnimplementedWebhookType(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    // --- Keep existing specific errors if needed, or map them to PayloadError ---
    #[error("Missing signature")]
    MissingSignature,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Webhook not configured")]
    WebhookNotConfigured,
    // --- End existing specific errors ---
}

impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            WebhookError::PayloadError(msg) => (StatusCode::BAD_REQUEST, msg),
            WebhookError::ActionExecutionError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            WebhookError::UnimplementedWebhookType(msg) => (StatusCode::NOT_IMPLEMENTED, msg),
            WebhookError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            // --- Map existing errors ---
            WebhookError::MissingSignature => (StatusCode::UNAUTHORIZED, "Missing signature".to_string()),
            WebhookError::InvalidSignature => (StatusCode::UNAUTHORIZED, "Invalid signature".to_string()),
            WebhookError::WebhookNotConfigured => (StatusCode::NOT_FOUND, "Webhook not configured".to_string()),
        };

        let body = Json(json!({"error": error_message}));

        (status, body).into_response()
    }
}

// Optional: Implement From<WebhookDataError> for WebhookError for easier conversion
impl From<WebhookDataError> for WebhookError {
    fn from(err: WebhookDataError) -> Self {
        WebhookError::PayloadError(err.to_string())
    }
}
]]>
