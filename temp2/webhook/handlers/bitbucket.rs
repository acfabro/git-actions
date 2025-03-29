use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::any::Any;
use reqwest::Client;

use crate::webhook::handlers::webhook_type::{WebhookTypeHandler, WebhookTypeHandlerBase, ApiConfig, AuthConfig};
use crate::webhook::error::WebhookError;
use crate::webhook::event::Event;

/// Handler for Bitbucket webhooks
pub struct BitbucketHandler {
    client: Client,
}

impl BitbucketHandler {
    /// Create a new BitbucketHandler
    pub fn new(_api_config: &ApiConfig) -> Self {
        Self {
            client: Client::new(),
        }
    }
    
    /// Extract repository information from payload
    fn extract_repo(&self, payload: &Value) -> Option<String> {
        payload["repository"]["slug"].as_str().map(|s| s.to_string())
    }
    
    /// Extract commit hash from payload
    fn extract_commit(&self, payload: &Value) -> Option<String> {
        payload["commit"]["hash"].as_str().map(|s| s.to_string())
    }
    
    /// Get changed files from payload or API
    async fn get_changed_files(&self, payload: &Value, api_config: &ApiConfig) -> Result<Vec<String>, WebhookError> {
        let mut paths = Vec::new();
        
        // Try to extract from payload first
        if let Some(changes) = payload["changes"].as_array() {
            for change in changes {
                if let Some(path) = change["path"].as_object().and_then(|p| p["toString"].as_str()) {
                    paths.push(path.to_string());
                }
            }
        }
        
        // If no paths found, try to get from API
        if paths.is_empty() {
            if let (Some(repo), Some(commit)) = (self.extract_repo(payload), self.extract_commit(payload)) {
                // Build request with API configuration
                let url = format!("{}/projects/{}/repos/{}/commits/{}/changes", 
                    api_config.base_url, 
                    api_config.project.as_deref().unwrap_or(""),
                    repo,
                    commit);
                
                let mut request = self.client.get(&url);
                
                // Add authentication
                match api_config.auth.auth_type.as_str() {
                    "token" => {
                        let token = api_config.auth.token.clone()
                            .or_else(|| {
                                api_config.auth.token_from_env.as_ref().and_then(|env_var| {
                                    std::env::var(env_var).ok()
                                })
                            })
                            .ok_or(WebhookError::MissingApiToken)?;
                        
                        request = request.header("Authorization", format!("Bearer {}", token));
                    },
                    "basic" => {
                        let username = api_config.auth.username.clone()
                            .or_else(|| {
                                api_config.auth.username_from_env.as_ref().and_then(|env_var| {
                                    std::env::var(env_var).ok()
                                })
                            })
                            .ok_or(WebhookError::MissingApiUsername)?;
                            
                        let password = api_config.auth.password.clone()
                            .or_else(|| {
                                api_config.auth.password_from_env.as_ref().and_then(|env_var| {
                                    std::env::var(env_var).ok()
                                })
                            })
                            .ok_or(WebhookError::MissingApiPassword)?;
                            
                        request = request.basic_auth(username, Some(password));
                    },
                    _ => return Err(WebhookError::UnsupportedAuthType(api_config.auth.auth_type.clone())),
                }
                
                // Add custom headers
                if let Some(headers) = &api_config.headers {
                    for (key, value) in headers {
                        request = request.header(key, value);
                    }
                }
                
                // Send request and process response
                let response = request.send().await
                    .map_err(|e| WebhookError::ExternalApiError(e.to_string()))?;
                    
                let response_body = response.json::<Value>().await
                    .map_err(|e| WebhookError::ExternalApiError(e.to_string()))?;
                    
                // Extract paths from response
                if let Some(values) = response_body["values"].as_array() {
                    for value in values {
                        if let Some(path) = value["path"]["toString"].as_str() {
                            paths.push(path.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(paths)
    }
}

impl WebhookTypeHandlerBase for BitbucketHandler {
    fn validate_webhook(&self, _payload: &Value, headers: &HashMap<String, String>, _secret: &str) -> Result<(), WebhookError> {
        // Bitbucket-specific signature validation
        if let Some(signature) = headers.get("x-hub-signature") {
            // TODO: Implement proper signature validation
            // For now, just check if the signature exists
            if signature.is_empty() {
                return Err(WebhookError::InvalidSignature);
            }
            
            // In a real implementation, we would validate the signature using HMAC
            // let hmac = hmac_sha1(secret.as_bytes(), payload);
            // if !constant_time_eq(hmac.as_bytes(), signature.as_bytes()) {
            //     return Err(WebhookError::InvalidSignature);
            // }
            
            Ok(())
        } else {
            // For development, allow missing signature
            tracing::warn!("Missing webhook signature, but allowing for development");
            Ok(())
            // In production, uncomment:
            // Err(WebhookError::MissingSignature)
        }
    }
    
    fn handler_type(&self) -> &'static str {
        "bitbucket"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl WebhookTypeHandler for BitbucketHandler {
    async fn parse_payload(&self, payload: &Value, _headers: &HashMap<String, String>, api_config: &ApiConfig) -> Result<Event, WebhookError> {
        // Extract event type
        let event_type = payload["eventKey"]
            .as_str()
            .ok_or(WebhookError::MissingEventType)?
            .to_string();
        
        // Extract branch
        let branch = if let Some(ref_change) = payload["refChanges"].as_array().and_then(|arr| arr.get(0)) {
            // Push event
            if let Some(branch) = ref_change["refId"].as_str() {
                Some(branch.replace("refs/heads/", ""))
            } else {
                None
            }
        } else if let Some(pr) = payload["pullRequest"].as_object() {
            // PR event
            if let Some(branch) = pr["fromRef"].as_object().and_then(|r| r["displayId"].as_str()) {
                Some(branch.to_string())
            } else {
                None
            }
        } else {
            None
        };
        
        // Get changed files (may involve API call)
        let changed_files = self.get_changed_files(payload, api_config).await?;
        
        Ok(Event::new(
            event_type,
            branch,
            changed_files,
            payload.clone(),
        ))
    }
}