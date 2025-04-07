pub mod common;
pub mod rules_config;
pub mod server_config;
pub mod webhook_config;

use anyhow::{Context, Result};
use glob::glob;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub use rules_config::{Rule, RulesConfig};
pub use server_config::ServerConfig;
    pub use webhook_config::WebhookConfig;

/// Configuration manager for Git-Actions
#[derive(Clone)]
pub struct Config {
    pub server_config: ServerConfig,
    pub webhooks: HashMap<String, WebhookConfig>,
    pub rules: HashMap<String, Rule>,
}

impl Config {
    /// Load configuration from a server config file
    pub fn load(server_config_path: &Path) -> Result<Self> {
        // Load server config
        let server_config = ServerConfig::from_file(&server_config_path.to_path_buf())?;
        
        // Load webhook configs
        let mut webhooks = HashMap::new();
        let mut rules = HashMap::new();
        
        for pattern in &server_config.spec.configs {
            // Resolve the pattern relative to the server config directory
            let base_dir = server_config_path.parent().unwrap_or_else(|| Path::new("."));
            let full_pattern = base_dir.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();
            
            // Handle glob patterns
            let paths = glob(&pattern_str)
                .with_context(|| format!("Failed to resolve glob pattern: {}", pattern_str))?;
                
            for path in paths {
                let path = path?;
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;
                
                // Determine the kind of configuration
                if content.contains("kind: WebhookConfig") {
                    let webhook_config: WebhookConfig = serde_yaml::from_str(&content)
                        .with_context(|| format!("Failed to parse webhook config: {}", path.display()))?;
                    
                    // Validate the webhook config
                    webhook_config.validate()
                        .map_err(|e| anyhow::anyhow!("Invalid webhook config {}: {}", path.display(), e))?;
                    
                    // Get the webhook name
                    let name = webhook_config.name()
                        .map_err(|e| anyhow::anyhow!("Invalid webhook config {}: {}", path.display(), e))?;
                    
                    webhooks.insert(name.to_string(), webhook_config);
                } else if content.contains("kind: RulesConfig") {
                    let rules_config: RulesConfig = serde_yaml::from_str(&content)
                        .with_context(|| format!("Failed to parse rules config: {}", path.display()))?;
                    
                    // Add rules to the map
                    rules.extend(rules_config.spec.rules);
                }
            }
        }
        
        Ok(Self {
            server_config,
            webhooks,
            rules,
        })
    }
}
