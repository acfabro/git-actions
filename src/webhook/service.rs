use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use crate::config::rules_config::Rule;
use crate::webhook::handlers::webhook_type::{WebhookTypeHandler, WebhookTypeHandlerBase, ApiConfig};
use crate::webhook::error::WebhookError;
use crate::webhook::rule_evaluator::RuleEvaluator;

/// Entry for a webhook in the service
pub struct WebhookEntry {
    pub webhook_name: String,
    pub path: String,
    pub secret_from_env: String,
    pub api_config: ApiConfig,
    pub handler: Box<dyn WebhookTypeHandlerBase>,
}

/// Service for processing webhooks
pub struct WebhookService {
    path_to_entry: HashMap<String, WebhookEntry>,
    name_to_entry: HashMap<String, Arc<WebhookEntry>>,
    rules: HashMap<String, Rule>,
    rule_evaluator: Arc<RuleEvaluator>,
}

impl WebhookService {
    /// Create a new WebhookService
    pub fn new(
        path_to_entry: HashMap<String, WebhookEntry>,
        rules: HashMap<String, Rule>,
    ) -> Self {
        // Create name to entry mapping
        let mut name_to_entry = HashMap::new();
        for (_, entry) in &path_to_entry {
            name_to_entry.insert(entry.webhook_name.clone(), Arc::new(entry.clone()));
        }
        
        // Create rule evaluator
        let rule_evaluator = Arc::new(RuleEvaluator::new());
        
        Self {
            path_to_entry,
            name_to_entry,
            rules,
            rule_evaluator,
        }
    }
    
    /// Get all webhook paths
    pub fn get_all_paths(&self) -> Vec<&str> {
        self.path_to_entry.keys().map(|s| s.as_str()).collect()
    }
    
    /// Process a webhook request
    pub async fn process_webhook(
        &self,
        path: &str,
        payload: &Value,
        headers: &HashMap<String, String>,
    ) -> Result<(), WebhookError> {
        // Get the webhook entry for this path
        let entry = self.path_to_entry.get(path)
            .ok_or_else(|| WebhookError::UnknownWebhookPath(path.to_string()))?;
        
        tracing::info!("Processing webhook: {} ({})", entry.webhook_name, path);
        
        // Validate the webhook
        entry.handler.validate_webhook(payload, headers, &entry.secret_from_env)?;
        
        // Parse the payload into a normalized event
        let event = match entry.handler.handler_type() {
            "bitbucket" => {
                // We know this is a BitbucketHandler
                use crate::webhook::handlers::bitbucket::BitbucketHandler;
                let handler = entry.handler.as_any().downcast_ref::<BitbucketHandler>()
                    .ok_or_else(|| WebhookError::InternalError("Failed to downcast handler".to_string()))?;
                handler.parse_payload(payload, headers, &entry.api_config).await?
            },
            handler_type => {
                return Err(WebhookError::UnsupportedWebhookType(handler_type.to_string()));
            }
        };
        
        tracing::info!("Event type: {}, Branch: {:?}", event.event_type, event.branch);
        tracing::info!("Changed files: {:?}", event.changed_files);
        
        // Find rules that apply to this webhook
        let mut applicable_rules = Vec::new();
        for (rule_id, rule) in &self.rules {
            if rule.webhooks.contains(&entry.webhook_name) {
                applicable_rules.push((rule_id, rule));
            }
        }
        
        tracing::info!("Found {} applicable rules", applicable_rules.len());
        
        // Evaluate applicable rules
        let mut matching_rules = Vec::new();
        for (rule_id, rule) in applicable_rules {
            if self.rule_evaluator.matches(&event, rule) {
                matching_rules.push((rule_id, rule));
            }
        }
        
        tracing::info!("Found {} matching rules", matching_rules.len());
        
        // Execute actions for matching rules
        for (rule_id, rule) in matching_rules {
            self.rule_evaluator.execute_actions(rule_id, rule, &event)?;
        }
        
        Ok(())
    }
}

// Clone implementation for WebhookEntry
impl Clone for WebhookEntry {
    fn clone(&self) -> Self {
        // This is a simplified clone implementation
        // In a real implementation, you would need to clone the handler properly
        unimplemented!("WebhookEntry clone is not implemented")
    }
}