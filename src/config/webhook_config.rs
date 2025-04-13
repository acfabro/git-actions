use serde::{Deserialize, Serialize};

use crate::config::common::{ApiVersion, Metadata};

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: ConfigKind,
    pub metadata: Metadata,
    pub spec: WebhookSpec,
}

// impl WebhookConfig {
//     /// Validate the webhook configuration
//     pub fn validate(&self) -> Result<(), String> {
//         // Check that metadata.name is present
//         if self.metadata.name.is_none() {
//             return Err("WebhookConfig must have metadata.name".to_string());
//         }
//
//         // Check that kind is WebhookConfig
//         if self.kind != "WebhookConfig" {
//             return Err(format!("Expected kind: WebhookConfig, got: {}", self.kind));
//         }
//
//         Ok(())
//     }
//
//     /// Get the name of the webhook
//     pub fn name(&self) -> Result<&str, String> {
//         self.metadata.name.as_deref()
//             .ok_or_else(|| "WebhookConfig must have metadata.name".to_string())
//     }
// }

/// Webhook specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSpec {
    /// Path of the webhook
    pub path: String,
    
    /// Type of the webhook
    #[serde(rename = "type")]
    pub webhook_type: String,

    /// bitbucket specific configuration
    pub bitbucket: Option<Bitbucket>,
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bitbucket {
    /// The base URL for the Bitbucket API
    pub api_base_url: String,

    /// The project key in Bitbucket
    pub api_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigKind {
    WebhookConfig,
    RulesConfig,
}

// impl From<WebhookApiConfig> for ApiConfig {
//     fn from(config: WebhookApiConfig) -> Self {
//         Self {
//             base_url: config.base_url,
//             project: config.project,
//             repo: config.repo,
//             owner: config.owner,
//             auth: config.auth.into(),
//             headers: config.headers,
//         }
//     }
// }
//
// impl From<WebhookAuthConfig> for AuthConfig {
//     fn from(config: WebhookAuthConfig) -> Self {
//         Self {
//             auth_type: config.auth_type,
//             token: config.token,
//             token_from_env: config.token_from_env,
//             username: config.username,
//             username_from_env: config.username_from_env,
//             password: config.password,
//             password_from_env: config.password_from_env,
//         }
//     }
// }