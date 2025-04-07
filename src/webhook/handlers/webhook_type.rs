use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::webhook::error::WebhookError;
use crate::webhook::event::Event;

use std::any::Any;

/// Base trait for webhook type handlers with sync methods
pub trait WebhookTypeHandlerBase: Send + Sync + Any {
    /// Validate the webhook signature
    fn validate_webhook(
        &self,
        payload: &Value,
        headers: &HashMap<String, String>,
        secret: &str,
    ) -> Result<(), WebhookError>;
    
    /// Get the handler type name
    fn handler_type(&self) -> &'static str;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Trait defining the interface for all webhook type handlers with async methods
#[async_trait]
pub trait WebhookTypeHandler: WebhookTypeHandlerBase {
    /// Parse the webhook payload into a normalized Event
    async fn parse_payload(
        &self,
        payload: &Value,
        headers: &HashMap<String, String>,
        api_config: &ApiConfig,
    ) -> Result<Event, WebhookError>;
}

/// API configuration for webhook handlers
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Base URL for API calls
    pub base_url: String,
    
    /// Project name (for Bitbucket)
    pub project: Option<String>,
    
    /// Repository name
    pub repo: Option<String>,
    
    /// Owner name (for GitHub)
    pub owner: Option<String>,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// Additional headers
    pub headers: Option<HashMap<String, String>>,
}

/// Authentication configuration for API calls
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Authentication type (token, basic, oauth)
    pub auth_type: String,
    
    /// Authentication token (if using token auth)
    pub token: Option<String>,
    
    /// Environment variable name containing the auth token
    pub token_from_env: Option<String>,
    
    /// Username for basic auth
    pub username: Option<String>,
    
    /// Environment variable name containing the username
    pub username_from_env: Option<String>,
    
    /// Password for basic auth
    pub password: Option<String>,
    
    /// Environment variable name containing the password
    pub password_from_env: Option<String>,
}