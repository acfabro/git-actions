use serde_json::Value;

/// Normalized event representation that all webhook types will convert their payloads into
#[derive(Debug, Clone)]
pub struct Event {
    /// Type of the event (e.g., "push", "pull_request")
    pub event_type: String,
    
    /// Branch name (if applicable)
    pub branch: Option<String>,
    
    /// List of files changed in the event
    pub changed_files: Vec<String>,
    
    /// Original payload for reference
    pub payload: Value,
}