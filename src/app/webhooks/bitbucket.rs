use std::collections::HashMap;
use async_trait::async_trait;
use crate::app::webhooks::types::WebhookTypeHandler;
use crate::config::Rule;
use serde_json::Value;
use tracing::debug;
use crate::Error;

pub struct Bitbucket<'a> {
    pub config: crate::config::webhook_config::Bitbucket,
    pub rules: HashMap<String, &'a Rule>,
    pub payload: Value,
}

#[async_trait]
impl WebhookTypeHandler for Bitbucket<'_> {
    async fn extract_event_type(&self) -> Option<String> {
        Some("push".to_string())
    }

    async fn extract_branch(&self) -> Option<String> {
        Some("release/1.0.0".to_string())
    }

    async fn extract_changed_files(&self) -> Vec<String> {
        Some(vec!["file1.txt".to_string(), "file2.txt".to_string()])
            .unwrap_or_default()
    }

    async fn matches_rule(&self, rule: &Rule) -> bool {
        todo!()
    }

    async fn run(&self) -> Result<(), Error> {
        // Implement the logic to process the webhook
        // This is where you would handle the webhook payload and apply rules
        for (rule_name, rule) in &self.rules {
            debug!("Processing rule: {}", rule_name);

            // Check paths rule
            let branch = self.extract_branch().await;



        }

        Ok(())
    }
}
