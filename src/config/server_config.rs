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
    /// Path to the log file. If not provided, logs will be written to stdout
    pub file: Option<PathBuf>,
}

impl Default for LoggingSpec {
    fn default() -> Self {
        Self {
            level: Some("INFO".to_string()),
            format: Some("text".to_string()),
            file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_deserialize() {
        let yaml = r#"
        apiVersion: git-actions/v1
        kind: ServerConfig
        metadata:
          name: "git-actions-server"
        spec:
          port: 8080
          host: 0.0.0.0
          tls:
            enabled: false
            cert_file: /path/to/cert.pem
            key_file: /path/to/key.pem
          logging:
            level: INFO
            format: json
            file: /var/log/git-actions.log
          configs:
            - "webhooks/*.yaml"
            - "rules.yaml"
        "#;

        let config: ServerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.api_version.to_string(), "git-actions/v1");
        assert_eq!(config.kind, "ServerConfig");
        assert_eq!(config.metadata.as_ref().unwrap().name.as_ref().unwrap(), "git-actions-server");
        assert_eq!(config.spec.port, 8080);
        assert_eq!(config.spec.host, "0.0.0.0");

        let tls = config.spec.tls.unwrap();
        assert_eq!(tls.enabled, false);
        assert_eq!(tls.cert_file, PathBuf::from("/path/to/cert.pem"));
        assert_eq!(tls.key_file, PathBuf::from("/path/to/key.pem"));

        let logging = config.spec.logging.unwrap();
        assert_eq!(logging.level, Some("INFO".to_string()));
        assert_eq!(logging.format, Some("json".to_string()));
        assert_eq!(logging.file, Some(PathBuf::from("/var/log/git-actions.log")));
        
        assert_eq!(config.spec.configs.len(), 2);
        assert_eq!(config.spec.configs[0], "webhooks/*.yaml");
        assert_eq!(config.spec.configs[1], "rules.yaml");
    }
}
