use crate::config::rules_config::Rule;
use crate::Error;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait WebhookTypeHandler: Send + Sync {
    /// Extract the event type from the payload
    async fn extract_event_type(&self) -> Option<String>;

    /// Extract the branch from the payload
    async fn extract_branch(&self) -> Option<String>;

    /// Extract changed files from the payload
    async fn extract_changed_files(&self) -> Vec<String>;

    /// Check if a rule matches the payload
    async fn matches_rule(&self, rule: &Rule) -> bool;

    /// Run the webhook handler
    async fn run(&self) -> Result<(), Error>;

}