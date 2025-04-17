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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bitbucket {
    /// Environment variable containing the token for webhook authentication
    #[serde(rename = "tokenFromEnv")]
    pub token_from_env: Option<String>,

    /// API configuration for Bitbucket
    pub api: BitbucketApi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_deserialize_webhook_config() {
        let yaml = r#"
apiVersion: v1
kind: Webhook
metadata:
  name: test-webhook
spec:
  path: "/webhook/bitbucket"
  bitbucket:
    tokenFromEnv: "BITBUCKET_WEBHOOK_TOKEN"
    api:
      baseUrl: "https://bitbucket.example.com/rest/api/1.0"
      project: "PROJECT"
      repo: "repo-name"
      auth:
        type: "token"
        tokenFromEnv: "BITBUCKET_API_TOKEN"
"#;

        let config: WebhookConfig = serde_yaml::from_str(yaml).unwrap();
        
        // Verify the config
        assert_eq!(config.api_version, "v1");
        assert!(matches!(config.kind, ConfigKind::Webhook));
        assert_eq!(config.metadata.name, "test-webhook");
        assert_eq!(config.spec.path, "/webhook/bitbucket");
        
        // Check bitbucket config
        let bitbucket = config.spec.bitbucket.unwrap();
        assert_eq!(bitbucket.token_from_env, Some("BITBUCKET_WEBHOOK_TOKEN".to_string()));
        
        // Check API config
        assert_eq!(bitbucket.api.base_url, "https://bitbucket.example.com/rest/api/1.0");
        assert_eq!(bitbucket.api.project, "PROJECT");
        assert_eq!(bitbucket.api.repo, "repo-name");
        
        // Check auth config
        assert_eq!(bitbucket.api.auth.auth_type, "token");
        assert_eq!(bitbucket.api.auth.token_from_env, "BITBUCKET_API_TOKEN");
    }

    #[test]
    fn test_deserialize_webhook_config_minimal() {
        let yaml = r#"
apiVersion: v1
kind: Webhook
metadata:
  name: minimal-webhook
spec:
  path: "/webhook/simple"
  bitbucket:
    api:
      baseUrl: "https://bitbucket.example.com/rest/api/1.0"
      project: "PROJECT"
      repo: "repo-name"
      auth:
        type: "token"
        tokenFromEnv: "BITBUCKET_API_TOKEN"
"#;

        let config: WebhookConfig = serde_yaml::from_str(yaml).unwrap();
        
        // Verify the config
        assert_eq!(config.api_version, "v1");
        assert!(matches!(config.kind, ConfigKind::Webhook));
        assert_eq!(config.metadata.name, "minimal-webhook");
        assert_eq!(config.spec.path, "/webhook/simple");
        
        // Check bitbucket config
        let bitbucket = config.spec.bitbucket.unwrap();
        assert_eq!(bitbucket.token_from_env, None);
    }

    #[test]
    fn test_deserialize_webhook_config_invalid() {
        let yaml = r#"
apiVersion: v1
kind: Webhook
metadata:
  name: invalid-webhook
spec:
  path: "/webhook/invalid"
  # Missing required bitbucket field
"#;

        let result: Result<WebhookConfig, _> = serde_yaml::from_str(yaml);
        
        // This should not fail because bitbucket is optional
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.spec.bitbucket, None);
    }
}