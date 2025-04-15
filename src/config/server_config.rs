use crate::config::common::{ApiVersion, Metadata};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Server configuration for Git-Actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: ApiVersion,
    pub kind: String,
    pub metadata: Option<Metadata>,
    pub spec: ServerSpec,
}

impl ServerConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let file_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Self = serde_yaml::from_str(&file_content)
            .with_context(|| "Failed to parse YAML configuration")?;

        Ok(config)
    }
}

/// Server specification containing HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSpec {
    pub port: u16,
    pub host: String,
    pub tls: Option<TlsSpec>,
    pub logging: Option<LoggingSpec>,
    /// Configuration files to load (webhooks and rules)
    pub configs: Vec<String>,
}

impl Default for ServerSpec {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            tls: None,
            logging: Some(LoggingSpec::default()),
            configs: Vec::new(),
        }
    }
}

/// TLS configuration for secure HTTP connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSpec {
    pub enabled: bool,
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSpec {
    pub level: Option<String>,
    pub format: Option<String>,
}

impl Default for LoggingSpec {
    fn default() -> Self {
        Self {
            level: Some("INFO".to_string()),
            format: Some("text".to_string()),
        }
    }
}
