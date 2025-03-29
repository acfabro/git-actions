use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::{fs, result};
use std::path::Path;
use thiserror::Error;
use crate::config::RulesConfig;

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
pub fn load_server_config<T, P>(path: P) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let file_content = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

    let config: T = serde_yaml::from_str(&file_content)
        .with_context(|| "Failed to parse YAML configuration: {}")?;

    Ok(config)
}

/// Load rule configurations from the paths specified in server config
pub fn load_rule_configs(paths: Option<Vec<String>>) -> result::Result<Vec<RulesConfig>, String> {
    let mut rule_configs = Vec::new();

    if let Some(rule_config_paths) = paths {
        for path in rule_config_paths {
            let rule_config = load_server_config::<RulesConfig, &Path>(Path::new(&path))
                .map_err(|err| format!("Failed to load rule config from {}: {}", path, err))?;

            rule_configs.push(rule_config);
        }
    }

    Ok(rule_configs)
}
