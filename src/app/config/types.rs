use super::{Rule, RulesConfig, WebhookConfig};
use anyhow::Result;
use anyhow::{bail, Context};
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum ConfigType {
    // Server(ServerConfig),
    Webhook(WebhookConfig),
    Rules(RulesConfig),
}

#[derive(Clone, Debug)]
pub struct Config {
    pub configs: Vec<ConfigType>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new Config instance
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Load configuration from a server config file
    pub fn load(&mut self, path: &Vec<String>) -> Result<()> {
        // Load all configs
        for pattern in path {
            // for each pattern handle glob patterns
            let paths = glob(pattern)
                .with_context(|| format!("Failed to resolve glob pattern: {}", pattern))?;

            for path in paths {
                let config = Self::load_path_config(&path?)?;
                self.configs.push(config);
            }
        }

        Ok(())
    }

    fn load_path_config(path: &PathBuf) -> Result<ConfigType> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let content =
            serde_yaml::from_str::<Value>(&content).with_context(|| "Failed to parse YAML")?;

        let empty = Value::String("".to_string());
        let kind = content.get("kind").unwrap_or(&empty);

        let config = match kind.as_str() {
            Some("Webhook") => {
                let webhook_config: WebhookConfig = serde_yaml::from_value(content.clone())
                    .with_context(|| {
                        format!("Failed to parse webhook config: {}", path.display())
                    })?;
                ConfigType::Webhook(webhook_config)
            }
            Some("Rules") => {
                let rules_config: RulesConfig = serde_yaml::from_value(content.clone())
                    .with_context(|| format!("Failed to parse rules config: {}", path.display()))?;
                ConfigType::Rules(rules_config)
            }
            _ => {
                bail!("Unknown config type: {:?}", kind);
            }
        };

        Ok(config)
    }

    pub fn find_webhook_by_path(&self, path: &str) -> Result<&WebhookConfig> {
        for config in &self.configs {
            if let ConfigType::Webhook(webhook_config) = config {
                if webhook_config.spec.path == path {
                    return Ok(webhook_config);
                }
            }
        }
        bail!("Webhook not found for path: {}", path);
    }

    pub fn find_rules_by_webhook(&self, webhook_name: &str) -> Result<HashMap<String, &Rule>> {
        let mut rules = HashMap::new();

        for config in &self.configs {
            if let ConfigType::Rules(rules_config) = config {
                for (name, rule) in rules_config.spec.rules.iter() {
                    let rule_name = format!("{}-{}", rules_config.metadata.name, name);
                    if rule.webhooks.contains(&webhook_name.to_string()) {
                        rules.insert(rule_name, rule);
                    }
                }
            }
        }

        Ok(rules)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigKind {
    Webhook,
    Rules,
}

/// API version for configuration files
pub type ApiVersion = String;

/// Common metadata for configuration resources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// Name of the resource
    pub name: String,
}
