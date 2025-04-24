use crate::app::config::rules::Action;
use crate::app::config::Rule;
use crate::app::webhooks::rule_evaluator;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use strum_macros::{AsRefStr, Display};
use tracing::debug;

#[async_trait]
pub trait WebhookTypeHandler: Send + Sync {
    /// Extract the event from the payload
    async fn extract_event(&self) -> Result<Event>;

    /// Run the webhook handler
    async fn run(&self) -> Result<Vec<&Action>>;

    ///
    fn evaluate_rules<'a>(event: &Event, rules: &HashMap<String, &'a Rule>) -> Vec<&'a Action> {
        let mut actions: Vec<&Action> = Vec::new();
        // This is where you would handle the webhook payload and apply rules
        for (rule_name, rule) in rules {
            // call matches_rule to check if the rule applies
            let result = rule_evaluator::check(event, rule);
            if result {
                debug!("OK Rule {}", rule_name);
                for action in &rule.actions {
                    actions.push(action);
                }
            } else {
                debug!("FAIL Rule {}", rule_name);
            }
        }

        actions
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub event_type: EventType,
    pub branch: Branch,
    pub changed_files: Vec<Path>,
}

pub type Branch = String;
pub type Path = String;

#[derive(Clone, Debug, PartialEq, AsRefStr, Display)]
pub enum EventType {
    #[strum(serialize = "pr_created")]
    PROpened,
    #[strum(serialize = "pr_modified")]
    PRModified,
    #[strum(serialize = "pr_merged")]
    PRMerged,
    // TODO add more event types as needed
}

impl TryFrom<&str> for EventType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pr:opened" => Ok(EventType::PROpened),
            "pr:modified" => Ok(EventType::PRModified),
            "pr:merged" => Ok(EventType::PRMerged),
            _ => Err(anyhow!("Invalid event type: {}", value)),
        }
    }
}