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
    pub working_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_deserialize_rule() {
        let yaml = r#"
description: "Test rule"
webhooks:
  - "test-webhook"
event_types:
  - "pr_created"
branches:
  - exact: "main"
  - pattern: "feature/*"
  - regex: "hotfix/.*"
paths:
  - exact: "src/main.rs"
  - pattern: "src/*.rs"
  - regex: "docs/.*\\.md"
actions:
  - http:
      method: "GET"
      url: "https://example.com"
      headers:
        Content-Type: "application/json"
      body: '{"key": "value"}'
  - shell:
      command: "echo 'Hello, world!'"
      working_dir: "/tmp"
"#;

        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        
        // Verify the rule
        assert_eq!(rule.description, Some("Test rule".to_string()));
        assert_eq!(rule.webhooks, vec!["test-webhook"]);
        assert_eq!(rule.event_types, Some(vec!["pr_created".to_string()]));
        
        // Check branch filters
        assert_eq!(rule.branches.as_ref().unwrap().len(), 3);
        if let BranchFilter::Exact { exact } = &rule.branches.as_ref().unwrap()[0] {
            assert_eq!(exact, "main");
        } else {
            panic!("Expected Exact branch filter");
        }
        
        if let BranchFilter::Pattern { pattern } = &rule.branches.as_ref().unwrap()[1] {
            assert_eq!(pattern, "feature/*");
        } else {
            panic!("Expected Pattern branch filter");
        }
        
        if let BranchFilter::Regex { regex } = &rule.branches.as_ref().unwrap()[2] {
            assert_eq!(regex, "hotfix/.*");
        } else {
            panic!("Expected Regex branch filter");
        }
        
        // Check path filters
        assert_eq!(rule.paths.as_ref().unwrap().len(), 3);
        if let PathFilter::Exact { exact } = &rule.paths.as_ref().unwrap()[0] {
            assert_eq!(exact, "src/main.rs");
        } else {
            panic!("Expected Exact path filter");
        }
        
        if let PathFilter::Pattern { pattern } = &rule.paths.as_ref().unwrap()[1] {
            assert_eq!(pattern, "src/*.rs");
        } else {
            panic!("Expected Pattern path filter");
        }
        
        if let PathFilter::Regex { regex } = &rule.paths.as_ref().unwrap()[2] {
            assert_eq!(regex, "docs/.*\\.md");
        } else {
            panic!("Expected Regex path filter");
        }
        
        // Check actions
        assert_eq!(rule.actions.len(), 2);
        
        // Check HTTP action
        let http_action = rule.actions[0].http.as_ref().unwrap();
        assert_eq!(http_action.method, "GET");
        assert_eq!(http_action.url, "https://example.com");
        assert_eq!(http_action.headers.as_ref().unwrap().get("Content-Type").unwrap(), "application/json");
        assert_eq!(http_action.body.as_ref().unwrap(), "{\"key\": \"value\"}");
        
        // Check shell action
        let shell_action = rule.actions[1].shell.as_ref().unwrap();
        assert_eq!(shell_action.command, "echo 'Hello, world!'");
        assert_eq!(shell_action.working_dir.as_ref().unwrap(), "/tmp");
    }

    #[test]
    fn test_deserialize_rules_config() {
        let yaml = r#"
apiVersion: v1
kind: Rules
metadata:
  name: test-rules
spec:
  rules:
    rule1:
      description: "Rule 1"
      webhooks:
        - "webhook1"
      event_types:
        - "pr_created"
      actions:
        - http:
            method: "GET"
            url: "https://example.com/1"
    rule2:
      description: "Rule 2"
      webhooks:
        - "webhook2"
      event_types:
        - "pr_modified"
      actions:
        - http:
            method: "POST"
            url: "https://example.com/2"
"#;

        let config: RulesConfig = serde_yaml::from_str(yaml).unwrap();
        
        // Verify the config
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.metadata.name, "test-rules");
        
        // Check rules
        assert_eq!(config.spec.rules.len(), 2);
        assert!(config.spec.rules.contains_key("rule1"));
        assert!(config.spec.rules.contains_key("rule2"));
        
        // Check rule1
        let rule1 = &config.spec.rules["rule1"];
        assert_eq!(rule1.description, Some("Rule 1".to_string()));
        assert_eq!(rule1.webhooks, vec!["webhook1"]);
        assert_eq!(rule1.event_types, Some(vec!["pr_created".to_string()]));
        assert_eq!(rule1.actions.len(), 1);
        
        // Check rule2
        let rule2 = &config.spec.rules["rule2"];
        assert_eq!(rule2.description, Some("Rule 2".to_string()));
        assert_eq!(rule2.webhooks, vec!["webhook2"]);
        assert_eq!(rule2.event_types, Some(vec!["pr_modified".to_string()]));
        assert_eq!(rule2.actions.len(), 1);
    }
}
