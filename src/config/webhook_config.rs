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
    /// The base URL for the Bitbucket API
    pub api_base_url: String,

    /// The project key in Bitbucket
    pub api_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigKind {
    Webhook,
    Rules,
}
