use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::common::{ApiVersion, Metadata};
use crate::webhook::handlers::webhook_type::{ApiConfig, AuthConfig};

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: WebhookSpec,
}

impl WebhookConfig {
    /// Validate the webhook configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check that metadata.name is present
        if self.metadata.name.is_none() {
            return Err("WebhookConfig must have metadata.name".to_string());
        }
        
        // Check that kind is WebhookConfig
        if self.kind != "WebhookConfig" {
            return Err(format!("Expected kind: WebhookConfig, got: {}", self.kind));
        }
        
        Ok(())
    }
    
    /// Get the name of the webhook
    pub fn name(&self) -> Result<&str, String> {
        self.metadata.name.as_deref()
            .ok_or_else(|| "WebhookConfig must have metadata.name".to_string())
    }
}

/// Webhook specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSpec {
    /// Path of the webhook
    pub path: String,
    
    /// Type of the webhook
    #[serde(rename = "type")]
    pub webhook_type: String,
    
    /// Environment variable name containing the webhook secret
    #[serde(rename = "secretFromEnv")]
    pub secret_from_env: String,
    
    /// API configuration
    pub api: WebhookApiConfig,
}

/// API configuration for webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookApiConfig {
    /// Base URL for API calls
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    
    /// Project name (for Bitbucket)
    pub project: Option<String>,
    
    /// Repository name
    pub repo: Option<String>,
    
    /// Owner name (for GitHub)
    pub owner: Option<String>,
    
    /// Authentication configuration
    pub auth: WebhookAuthConfig,
    
    /// Additional headers
    pub headers: Option<HashMap<String, String>>,
}

/// Authentication configuration for webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAuthConfig {
    /// Authentication type (token, basic, oauth)
    #[serde(rename = "type")]
    pub auth_type: String,
    
    /// Authentication token (if using token auth)
    pub token: Option<String>,
    
    /// Environment variable name containing the auth token
    #[serde(rename = "tokenFromEnv")]
    pub token_from_env: Option<String>,
    
    /// Username for basic auth
    pub username: Option<String>,
    
    /// Environment variable name containing the username
    #[serde(rename = "usernameFromEnv")]
    pub username_from_env: Option<String>,
    
    /// Password for basic auth
    pub password: Option<String>,
    
    /// Environment variable name containing the password
    #[serde(rename = "passwordFromEnv")]
    pub password_from_env: Option<String>,
}

impl From<WebhookApiConfig> for ApiConfig {
    fn from(config: WebhookApiConfig) -> Self {
        Self {
            base_url: config.base_url,
            project: config.project,
            repo: config.repo,
            owner: config.owner,
            auth: config.auth.into(),
            headers: config.headers,
        }
    }
}

impl From<WebhookAuthConfig> for AuthConfig {
    fn from(config: WebhookAuthConfig) -> Self {
        Self {
            auth_type: config.auth_type,
            token: config.token,
            token_from_env: config.token_from_env,
            username: config.username,
            username_from_env: config.username_from_env,
            password: config.password,
            password_from_env: config.password_from_env,
        }
    }
}