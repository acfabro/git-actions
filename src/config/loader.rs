use crate::config::{RulesConfig, ServerConfig};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::info;

/// Errors that can occur when loading configuration
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),
}

/// Loads configuration from a YAML file
pub fn load_server_config(path: PathBuf) -> Result<ServerConfig>
{
    let file_content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {:?}", &path))?;

    let config = serde_yaml::from_str(&file_content)
        .with_context(|| "Failed to parse YAML configuration: {}")?;

    Ok(config)
}

/// Load rule configurations from the paths specified in server config
pub fn load_rule_configs(paths: Vec<String>) -> Result<Vec<RulesConfig>> {
    let mut rule_configs = Vec::new();

    // Iterate over each path and load the corresponding configuration
    for path in paths {
        info!("Loading rules config: {}", &path);

        let file_content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read rule config file: {:?}", &path))?;

        let rule_config: RulesConfig = serde_yaml::from_str(&file_content)
            .with_context(|| format!("Failed to parse YAML configuration: {:?}", &path))?;

        // Validate the loaded configuration
        if rule_config.spec.rules.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "No rules found in configuration file: {:?}",
                path
            ))
            .into());
        }

        rule_configs.push(rule_config);
    }

    Ok(rule_configs)
}
