pub mod error;
pub mod handlers;
pub mod router;
pub mod server;

use crate::config::{rules_config::{BranchFilter, PathFilter, Rule}, RulesConfig, ServerConfig};
use crate::logging;
use anyhow::{Context, Result};
use serde_json::Value;
use server::Server;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub rule_matcher: Arc<RuleMatcher>,
}

/// RuleMatcher service for finding and executing rules
pub struct RuleMatcher {
    // Map webhook paths directly to their applicable rules
    path_to_rules: HashMap<String, HashMap<String, Rule>>,
}

impl RuleMatcher {
    /// Create a new RuleMatcher from loaded configurations
    pub fn new(rule_configs: &Vec<RulesConfig>) -> Self {
        let mut path_to_rules = HashMap::new();

        for config in rule_configs {
            // For each webhook in this config
            for (_, webhook) in &config.spec.webhooks {
                // Map this webhook path to the rules in this config
                path_to_rules.insert(webhook.path.clone(), config.spec.rules.clone());
            }
        }

        Self { path_to_rules }
    }

    /// Get all rules associated with a specific webhook path
    pub fn get_rules_for_path(&self, path: &str) -> Option<&HashMap<String, Rule>> {
        self.path_to_rules.get(path)
    }

    /// Find rules that match the given criteria for a specific webhook path
    pub fn find_matching_rules(&self, path: &str, event_type: &str, branch: &str, paths: &[String]) -> Vec<(&String, &Rule)> {
        if let Some(rules) = self.get_rules_for_path(path) {
            return rules.iter()
                .filter(|(_rule_name, rule)| {
                    // Match event type (if specified)
                    if let Some(event_types) = &rule.event_types {
                        if !event_types.contains(&event_type.to_string()) {
                            return false;
                        }
                    }

                    // Match branch (if specified)
                    if let Some(branches) = &rule.branches {
                        let branch_matches = branches.iter().any(|filter| {
                            match filter {
                                BranchFilter::Exact { exact } => exact == branch,
                                BranchFilter::Pattern { pattern } => matches_pattern(branch, pattern),
                                BranchFilter::Regex { regex } => matches_regex(branch, regex),
                            }
                        });

                        if !branch_matches {
                            return false;
                        }
                    }

                    // Match paths (if specified and if any paths were modified)
                    if !paths.is_empty() {
                        if let Some(path_filters) = &rule.paths {
                            let path_matches = paths.iter().any(|modified_path| {
                                path_filters.iter().any(|filter| {
                                    match filter {
                                        PathFilter::Exact { exact } => exact == modified_path,
                                        PathFilter::Pattern { pattern } => matches_pattern(modified_path, pattern),
                                        PathFilter::Regex { regex } => matches_regex(modified_path, regex),
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
                })
                .collect();
        }

        // No rules found for the path
        Vec::new()
    }

    /// Execute actions for the given matching rules
    pub fn execute_actions(&self, rules: Vec<(&String, &Rule)>, _payload: &Value) -> Result<(), anyhow::Error> {
        for (rule_name, rule) in rules {
            tracing::info!("Executing actions for rule: {}", rule_name);
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
        }

        Ok(())
    }
}

// Helper functions for pattern and regex matching (placeholders)
// TODO: Implement proper pattern and regex matching using crates like `globset` and `regex`
fn matches_pattern(value: &str, pattern: &str) -> bool {
    debug!("Matching pattern: value='{}', pattern='{}'", value, pattern);
    // Placeholder implementation
    if pattern.ends_with('*') {
        value.starts_with(&pattern[..pattern.len() - 1])
    } else {
        value == pattern
    }
}

fn matches_regex(value: &str, regex_str: &str) -> bool {
    debug!("Matching regex: value='{}', regex='{}'", value, regex_str);
    // Placeholder implementation using basic contains for now
    value.contains(regex_str)
    // In a real implementation, use the regex crate:
    // match regex::Regex::new(regex_str) {
    //     Ok(re) => re.is_match(value),
    //     Err(e) => {
    //         tracing::error!("Invalid regex '{}': {}", regex_str, e);
    //         false
    //     }
    // }
}


/// Run the HTTP server with the given configuration
pub async fn run(config: ServerConfig) -> Result<()> {
    // Set up logging based on configuration
    logging::setup(&config.spec.logging)
        .with_context(|| "Failed to setup logging")?;

    // Create HTTP server
    let mut server = Server::new(&config);

    if let Some(tls) = &config.spec.tls {
        if tls.enabled {
            info!("TLS is enabled with certificate: {:?}", tls.cert_file);
        }
    }

    server.start().await?;

    Ok(())
}
