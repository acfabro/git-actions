use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::common::{ApiVersion, Metadata};
use crate::config::webhook_config::ConfigKind;

/// Rules configuration for Git-Actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// API version of the configuration
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    
    /// Kind of the configuration
    pub kind: ConfigKind,
    
    /// Metadata for the configuration
    pub metadata: Metadata,
    
    /// Configuration specification
    pub spec: RulesSpec,
}

/// Rules specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesSpec {
    /// Rules configurations - hashmap with rule name as key
    pub rules: HashMap<String, Rule>,
}

/// Rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Description of the rule
    pub description: Option<String>,
    
    /// Webhooks this rule applies to
    pub webhooks: Vec<String>,
    
    /// Event types to match
    #[serde(rename = "event_types")]
    pub event_types: Option<Vec<String>>,
    
    /// Branch filters to match
    pub branches: Option<Vec<BranchFilter>>,
    
    /// Path filters to match
    pub paths: Option<Vec<PathFilter>>,
    
    /// Actions to perform when the rule matches
    pub actions: Vec<Action>,
}

/// Branch filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BranchFilter {
    /// Exact branch name match
    Exact { exact: String },
    
    /// Pattern branch name match
    Pattern { pattern: String },
    
    /// Regex branch name match
    Regex { regex: String },

    // TODO: Implement negation logic
    // Not { not: String },
}

/// Path filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathFilter {
    /// Exact path match
    Exact { exact: String },
    
    /// Pattern path match
    Pattern { pattern: String },
    
    /// Regex path match
    Regex { regex: String },

    // TODO: Implement negation logic
    // Not { not: String },
}

/// Action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// HTTP action
    pub http: Option<HttpAction>,
    
    /// Shell action
    pub shell: Option<ShellAction>,
}

/// HTTP action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAction {
    /// HTTP method
    pub method: String,
    
    /// URL to call
    pub url: String,
    
    /// HTTP headers
    pub headers: Option<HashMap<String, String>>,
    
    /// HTTP body
    pub body: Option<String>,
}

/// Shell action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAction {
    /// Command to execute
    pub command: String,
    
    /// Working directory
    pub working_dir: String,
}
