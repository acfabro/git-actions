use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use thiserror::Error;

/// Errors that can occur during webhook processing
#[derive(Debug, Error)]
pub enum WebhookError {
    /// The webhook path was not found in the configuration
    #[error("Unknown webhook path: {0}")]
    UnknownWebhookPath(String),

    /// The webhook type is not supported
    #[error("Unsupported webhook type: {0}")]
    UnsupportedWebhookType(String),

    /// The webhook signature is missing
    #[error("Missing webhook signature")]
    MissingSignature,

    /// The webhook signature is invalid
    #[error("Invalid webhook signature")]
    InvalidSignature,

    /// The event type is missing from the payload
    #[error("Missing event type in webhook payload")]
    MissingEventType,

    /// The API token is missing
    #[error("Missing API token")]
    MissingApiToken,

    /// The API username is missing
    #[error("Missing API username")]
    MissingApiUsername,

    /// The API password is missing
    #[error("Missing API password")]
    MissingApiPassword,

    /// The authentication type is not supported
    #[error("Unsupported authentication type: {0}")]
    UnsupportedAuthType(String),

    /// An error occurred while calling an external API
    #[error("External API error: {0}")]
    ExternalApiError(String),

    /// No rules were found for the webhook path
    #[error("No rules found for path: {0}")]
    NoRulesForPath(String),

    /// No handler was found for the webhook path
    #[error("No handler found for path: {0}")]
    NoHandlerForPath(String),

    /// An error occurred while executing actions
    #[error("Action execution error: {0}")]
    ActionExecutionError(String),

    /// An error occurred while parsing the webhook payload
    #[error("Payload parsing error: {0}")]
    PayloadParsingError(String),

    /// An error occurred while serializing the payload
    #[error("Payload serialization error: {0}")]
    PayloadSerializationError(String),

    /// An error occurred in the configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// An error occurred while loading the configuration
    #[error("Configuration loading error: {0}")]
    ConfigLoadingError(#[from] std::io::Error),

    /// An error occurred while parsing the configuration
    #[error("Configuration parsing error: {0}")]
    ConfigParsingError(#[from] serde_yaml::Error),

    /// An error occurred while validating the configuration
    #[error("Configuration validation error: {0}")]
    ConfigValidationError(String),

    /// An error occurred while executing an action
    #[error("Action error: {0}")]
    ActionError(String),

    /// An error occurred while executing a rule
    #[error("Rule error: {0}")]
    RuleError(String),

    /// A generic webhook error
    #[error("{0}")]
    Other(String),
    
    /// An internal error occurred
    #[error("Internal error: {0}")]
    InternalError(String),
}

// Implement IntoResponse for WebhookError
impl IntoResponse for WebhookError {
    fn into_response(self) -> Response {
        let status = match self {
            WebhookError::UnknownWebhookPath(_) => StatusCode::NOT_FOUND,
            WebhookError::UnsupportedWebhookType(_) => StatusCode::BAD_REQUEST,
            WebhookError::MissingSignature => StatusCode::UNAUTHORIZED,
            WebhookError::InvalidSignature => StatusCode::UNAUTHORIZED,
            WebhookError::MissingEventType => StatusCode::BAD_REQUEST,
            WebhookError::MissingApiToken => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::MissingApiUsername => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::MissingApiPassword => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::UnsupportedAuthType(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::ExternalApiError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::NoRulesForPath(_) => StatusCode::NOT_FOUND,
            WebhookError::NoHandlerForPath(_) => StatusCode::NOT_FOUND,
            WebhookError::ActionExecutionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::PayloadParsingError(_) => StatusCode::BAD_REQUEST,
            WebhookError::PayloadSerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::ConfigLoadingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::ConfigParsingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::ConfigValidationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::ActionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::RuleError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebhookError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = self.to_string();
        
        (status, body).into_response()
    }
}