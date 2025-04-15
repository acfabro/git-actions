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

/// Webhook specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSpec {
    /// Path of the webhook
    pub path: String,

    /// bitbucket specific configuration
    pub bitbucket: Option<Bitbucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bitbucket {
    /// Environment variable containing the token for webhook authentication
    #[serde(rename = "tokenFromEnv")]
    pub token_from_env: Option<String>,

    /// API configuration for Bitbucket
    pub api: BitbucketApi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketApi {
    /// The base URL for the Bitbucket API
    #[serde(rename = "baseUrl")]
    pub base_url: String,

    /// The project key/slug in Bitbucket
    pub project: String,

    /// The repository name/slug in Bitbucket
    pub repo: String,

    /// Authentication configuration for Bitbucket API
    pub auth: BitbucketAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketAuth {
    /// Authentication type (token, basic)
    #[serde(rename = "type")]
    pub auth_type: String,

    /// Environment variable containing the API token
    #[serde(rename = "tokenFromEnv")]
    pub token_from_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigKind {
    Webhook,
    Rules,
}

impl WebhookConfig {
    // TODO: Implement validation logic
    // pub fn validate() {
    // }
}