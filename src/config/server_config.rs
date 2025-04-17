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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_server_config_from_file() {
        // Create a temporary file with valid server config
        let mut file = NamedTempFile::new().unwrap();
        let config_content = r#"
apiVersion: v1
kind: Server
metadata:
  name: test-server
spec:
  host: 127.0.0.1
  port: 8080
  logging:
    level: info
    format: json
  configs:
    - "rules.yaml"
    - "webhooks.yaml"
"#;
        file.write_all(config_content.as_bytes()).unwrap();

        // Load the config from the file
        let config = ServerConfig::from_file(&file.path().to_path_buf()).unwrap();

        // Verify the config
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.kind, "Server");
        assert!(config.metadata.is_some());
        if let Some(metadata) = config.metadata {
            assert_eq!(metadata.name, "test-server");
        }
        assert_eq!(config.spec.host, "127.0.0.1");
        assert_eq!(config.spec.port, 8080);
        assert_eq!(config.spec.configs.len(), 2);
        assert_eq!(config.spec.configs[0], "rules.yaml");
        assert_eq!(config.spec.configs[1], "webhooks.yaml");
    }

    #[test]
    fn test_server_config_from_file_invalid_yaml() {
        // Create a temporary file with invalid YAML
        let mut file = NamedTempFile::new().unwrap();
        let config_content = r#"
apiVersion: v1
kind: Server
metadata:
  name: test-server
spec:
  host: 127.0.0.1
  port: not-a-number  # This should cause a parsing error
  logging:
    level: info
    format: json
  configs:
    - "rules.yaml"
"#;
        file.write_all(config_content.as_bytes()).unwrap();

        // Try to load the config from the file
        let result = ServerConfig::from_file(&file.path().to_path_buf());

        // Verify that it returns an error
        assert!(result.is_err());
    }

    #[test]
    fn test_server_config_from_file_nonexistent() {
        // Try to load config from a nonexistent file
        let result = ServerConfig::from_file(&PathBuf::from("nonexistent-file.yaml"));

        // Verify that it returns an error
        assert!(result.is_err());
    }
}

