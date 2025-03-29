// use thiserror::Error;
//
// use crate::config::common::{ServerConfig, TlsConfig};
//
// /// Errors that can occur during configuration validation
// #[derive(Debug, Error)]
// pub enum ValidationError {
//     #[error("Invalid port number: {0}")]
//     InvalidPort(u16),
//
//     #[error("Invalid host: {0}")]
//     InvalidHost(String),
//
//     #[error("Invalid TLS configuration: {0}")]
//     InvalidTlsConfig(String),
//
//     #[error("Invalid logging configuration: {0}")]
//     InvalidLoggingConfig(String),
//
//     #[error("Invalid rule config path: {0}")]
//     InvalidRuleConfigPath(String),
// }
//
// /// Validates the server configuration
// pub fn validate_server_config(config: &ServerConfig) -> Result<(), ValidationError> {
//     // Validate port number (avoid well-known ports below 1024 unless explicitly needed)
//     if config.spec.port < 1024 {
//         return Err(ValidationError::InvalidPort(config.spec.port));
//     }
//
//     // Validate host
//     if config.spec.host.is_empty() {
//         return Err(ValidationError::InvalidHost(
//             "Host cannot be empty".to_string(),
//         ));
//     }
//
//     // Validate TLS configuration if enabled
//     if let Some(tls) = &config.spec.tls {
//         validate_tls_config(tls)?;
//     }
//
//     // Validate logging configuration
//     if let Some(logging) = &config.spec.logging {
//         // Validate log level if specified
//         if let Some(level) = &logging.level {
//             match level.to_uppercase().as_str() {
//                 "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE" => {}
//                 _ => {
//                     return Err(ValidationError::InvalidLoggingConfig(format!(
//                         "Invalid log level: {}",
//                         level
//                     )))
//                 }
//             }
//         }
//
//         // Validate log format if specified
//         if let Some(format) = &logging.format {
//             match format.to_lowercase().as_str() {
//                 "text" | "json" | "structured" => {}
//                 _ => {
//                     return Err(ValidationError::InvalidLoggingConfig(format!(
//                         "Invalid log format: {}",
//                         format
//                     )))
//                 }
//             }
//         }
//
//         // Validate log file path if specified
//         if let Some(file) = &logging.file {
//             if file.is_empty() {
//                 return Err(ValidationError::InvalidLoggingConfig(
//                     "Log file path cannot be empty".to_string(),
//                 ));
//             }
//         }
//     }
//
//     // Validate rule config paths
//     if let Some(rule_configs) = &config.spec.rule_configs {
//         for path in rule_configs {
//             if path.is_empty() {
//                 return Err(ValidationError::InvalidRuleConfigPath(
//                     "Rule config path cannot be empty".to_string(),
//                 ));
//             }
//         }
//     }
//
//     Ok(())
// }
//
// /// Validates TLS configuration
// fn validate_tls_config(tls: &TlsConfig) -> Result<(), ValidationError> {
//     if !tls.enabled {
//         return Ok(());
//     }
//
//     // Validate cert file
//     if tls.cert_file.is_empty() {
//         return Err(ValidationError::InvalidTlsConfig(
//             "Certificate file path cannot be empty when TLS is enabled".to_string(),
//         ));
//     }
//
//     // Validate key file
//     if tls.key_file.is_empty() {
//         return Err(ValidationError::InvalidTlsConfig(
//             "Key file path cannot be empty when TLS is enabled".to_string(),
//         ));
//     }
//
//     // Check if cert file exists (optional, can be commented out if not needed during validation)
//     // if !Path::new(&tls.cert_file).exists() {
//     //     return Err(ValidationError::InvalidTlsConfig(
//     //         format!("Certificate file does not exist: {}", tls.cert_file)
//     //     ));
//     // }
//
//     // Check if key file exists (optional, can be commented out if not needed during validation)
//     // if !Path::new(&tls.key_file).exists() {
//     //     return Err(ValidationError::InvalidTlsConfig(
//     //         format!("Key file does not exist: {}", tls.key_file)
//     //     ));
//     // }
//
//     Ok(())
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::config::common::{ApiVersion, LoggingConfig, ServerSpec};
//
//     fn create_test_config() -> ServerConfig {
//         ServerConfig {
//             api_version: ApiVersion::new("git-actions", "v1"),
//             kind: "ServerConfig".to_string(),
//             spec: ServerSpec {
//                 port: 8080,
//                 host: "0.0.0.0".to_string(),
//                 tls: None,
//                 logging: Some(LoggingConfig {
//                     level: Some("INFO".to_string()),
//                     format: Some("json".to_string()),
//                     file: None,
//                 }),
//                 rule_configs: None,
//             },
//         }
//     }
//
//     #[test]
//     fn test_valid_config() {
//         let config = create_test_config();
//         let result = validate_server_config(&config);
//         assert!(result.is_ok());
//     }
//
//     #[test]
//     fn test_invalid_port() {
//         let mut config = create_test_config();
//         config.spec.port = 80; // Reserved port
//         let result = validate_server_config(&config);
//         assert!(result.is_err());
//         match result {
//             Err(ValidationError::InvalidPort(port)) => assert_eq!(port, 80),
//             _ => panic!("Expected InvalidPort error"),
//         }
//     }
//
//     #[test]
//     fn test_invalid_host() {
//         let mut config = create_test_config();
//         config.spec.host = "".to_string();
//         let result = validate_server_config(&config);
//         assert!(result.is_err());
//         match result {
//             Err(ValidationError::InvalidHost(_)) => {}
//             _ => panic!("Expected InvalidHost error"),
//         }
//     }
//
//     #[test]
//     fn test_invalid_tls_config() {
//         let mut config = create_test_config();
//         config.spec.tls = Some(TlsConfig {
//             enabled: true,
//             cert_file: "".to_string(),
//             key_file: "/path/to/key.pem".to_string(),
//         });
//         let result = validate_server_config(&config);
//         assert!(result.is_err());
//         match result {
//             Err(ValidationError::InvalidTlsConfig(_)) => {}
//             _ => panic!("Expected InvalidTlsConfig error"),
//         }
//     }
//
//     #[test]
//     fn test_invalid_logging_level() {
//         let mut config = create_test_config();
//         if let Some(logging) = &mut config.spec.logging {
//             logging.level = Some("INVALID".to_string());
//         }
//         let result = validate_server_config(&config);
//         assert!(result.is_err());
//         match result {
//             Err(ValidationError::InvalidLoggingConfig(_)) => {}
//             _ => panic!("Expected InvalidLoggingConfig error"),
//         }
//     }
// }
