use std::collections::HashMap;
use serde_json::Value;

use crate::config::rules_config::{Rule, BranchFilter, PathFilter};
use crate::webhook::event::Event;
use crate::webhook::error::WebhookError;

/// Rule evaluator for matching rules against events
pub struct RuleEvaluator;

impl RuleEvaluator {
    /// Create a new RuleEvaluator
    pub fn new() -> Self {
        Self
    }
    
    /// Check if an event matches a rule
    pub fn matches(&self, event: &Event, rule: &Rule) -> bool {
        // Match event type (if specified)
        if let Some(event_types) = &rule.event_types {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }
        
        // Match branch (if specified)
        if let Some(branch_filters) = &rule.branches {
            if let Some(branch) = &event.branch {
                let branch_matches = branch_filters.iter().any(|filter| {
                    match filter {
                        BranchFilter::Exact { exact } => exact == branch,
                        BranchFilter::Pattern { pattern } => self.matches_pattern(branch, pattern),
                        BranchFilter::Regex { regex } => self.matches_regex(branch, regex),
                    }
                });
                
                if !branch_matches {
                    return false;
                }
            } else {
                // No branch information available
                return false;
            }
        }
        
        // Match paths (if specified and if any paths were modified)
        if !event.changed_files.is_empty() {
            if let Some(path_filters) = &rule.paths {
                let path_matches = event.changed_files.iter().any(|modified_path| {
                    path_filters.iter().any(|filter| {
                        match filter {
                            PathFilter::Exact { exact } => exact == modified_path,
                            PathFilter::Pattern { pattern } => self.matches_pattern(modified_path, pattern),
                            PathFilter::Regex { regex } => self.matches_regex(modified_path, regex),
                        }
                    })
                });
                
                if !path_matches {
                    return false;
                }
            }
        }
        
        // If all checks pass, the rule matches
        true
    }
    
    /// Execute actions for a matching rule
    pub fn execute_actions(&self, rule_id: &str, rule: &Rule, event: &Event) -> Result<(), WebhookError> {
        tracing::info!("Executing actions for rule: {}", rule_id);
        
        for action in &rule.actions {
            // Execute action based on type
            if let Some(http) = &action.http {
                // TODO: Implement HTTP action execution
                tracing::info!("Executing HTTP action: {} {}", http.method, http.url);
            } else if let Some(shell) = &action.shell {
                // TODO: Implement Shell action execution
                tracing::info!("Executing Shell action: {}", shell.command);
            }
        }
        
        Ok(())
    }
    
    /// Check if a value matches a pattern
    fn matches_pattern(&self, value: &str, pattern: &str) -> bool {
        // Simple pattern matching implementation
        if pattern.ends_with('*') {
            value.starts_with(&pattern[..pattern.len() - 1])
        } else {
            value == pattern
        }
    }
    
    /// Check if a value matches a regex
    fn matches_regex(&self, value: &str, regex_str: &str) -> bool {
        // Simple regex matching implementation
        // In a real implementation, use the regex crate
        value.contains(regex_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matches_pattern() {
        let evaluator = RuleEvaluator::new();
        
        assert!(evaluator.matches_pattern("feature/test", "feature/*"));
        assert!(evaluator.matches_pattern("feature/test", "feature/test"));
        assert!(!evaluator.matches_pattern("bugfix/test", "feature/*"));
    }
    
    #[test]
    fn test_matches_regex() {
        let evaluator = RuleEvaluator::new();
        
        assert!(evaluator.matches_regex("feature/test", "feature"));
        assert!(evaluator.matches_regex("feature/test", "test"));
        assert!(!evaluator.matches_regex("bugfix/test", "feature"));
    }
}