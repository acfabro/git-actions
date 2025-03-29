pub mod common;
pub mod rules_config;
pub mod server_config;
pub mod webhook_config;

pub use rules_config::{Rule, RulesConfig};
pub use server_config::ServerConfig;
pub use webhook_config::{WebhookConfig, WebhookSpec};

use anyhow::{Context, Result};
use glob::glob;
use std::collections::HashMap;
use std::fs;
use tracing::{debug, warn};

/// Configuration manager for Git-Actions
#[derive(Clone, Debug)]
pub struct Config {
    pub webhooks: HashMap<String, WebhookSpec>,
    pub rules: HashMap<String, Rule>,
}

impl Config {
    /// Load configuration from a server config file
    pub fn load(path: &Vec<String>) -> Result<Self> {
        // Load all configs
        let mut webhooks = HashMap::new();
        let mut rules = HashMap::new();
        for pattern in path {
            // for each pattern handle glob patterns
            let paths = glob(&pattern)
                .with_context(|| format!("Failed to resolve glob pattern: {}", pattern))?;

            for path in paths {
                let path = path?;
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

                // TODO: is there a better way?
                if content.contains("kind: Webhook") {
                    let webhook_config: WebhookConfig = serde_yaml::from_str(&content)
                        .with_context(|| {
                            format!("Failed to parse webhook config: {}", path.display())
                        })?;

                    debug!(
                        "Registering webhook [{}] [{}]",
                        webhook_config.metadata.name, webhook_config.spec.path,
                    );
                    webhooks.insert(webhook_config.metadata.name, webhook_config.spec);
                } else if content.contains("kind: Rules") {
                    let rules_config: RulesConfig =
                        serde_yaml::from_str(&content).with_context(|| {
                            format!("Failed to parse rules config: {}", path.display())
                        })?;

                    let set_rules = Self::extract_rules(&rules_config).with_context(|| {
                        format!("Failed to extract rules from config: {}", path.display())
                    })?;

                    debug!(
                        "Registered rules [{}] [{}]",
                        rules_config.metadata.name,
                        rules_config.spec.rules.len()
                    );
                    rules.extend(set_rules);
                } else {
                    warn!("Skipping config file with unknown type: {}", path.display());
                }
            }
        }

        Ok(Self { webhooks, rules })
    }

    fn extract_rules(rules_config: &RulesConfig) -> Result<HashMap<String, Rule>> {
        let mut rules = HashMap::new();
        for (name, rule) in rules_config.spec.rules.iter() {
            let rule_name = format!("{}-{}", rules_config.metadata.name, name);

            debug!("Registering rule [{}]", rule_name);
            rules.insert(rule_name, rule.clone());
        }
        Ok(rules)
    }
}
