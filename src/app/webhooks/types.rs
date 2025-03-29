use crate::Error;
use async_trait::async_trait;
use strum_macros::{AsRefStr, Display};
use crate::config::rules_config::Action;

#[async_trait]
pub trait WebhookTypeHandler: Send + Sync {
    /// Extract the event type from the payload
    async fn extract_event_type(&self) -> Result<EventType, Error>;

    /// Extract the branch from the payload
    async fn extract_branch(&self) -> Result<String, Error>;

    /// Extract changed files from the payload
    async fn extract_changed_files(&self) -> Result<Vec<String>, Error>;

    /// Run the webhook handler
    async fn run(&self) -> Result<Vec<&Action>, Error>;
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
    PRCreated,
    #[strum(serialize = "pr_modified")]
    PRModified,
    #[strum(serialize = "pr_merged")]
    PRMerged,
    // TODO add more event types as needed
}
