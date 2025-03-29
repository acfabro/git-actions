use crate::config::server_config::LoggingSpec;
use anyhow::Result;
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter};

/// Sets up logging based on the server configuration
pub fn setup(config: &Option<LoggingSpec>) -> Result<()> {
    let level_str = config
        .as_ref()
        .and_then(|c| c.level.as_ref())
        .unwrap_or(&"INFO".to_string())
        .to_uppercase();

    let format_str = config
        .as_ref()
        .and_then(|c| c.format.as_ref())
        .unwrap_or(&"text".to_string())
        .to_lowercase();

    // Set up the subscriber
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&level_str));

    // Configure the subscriber based on the format
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter);

    // If a log file is specified, use it
    if let Some(file_path) = config.as_ref().and_then(|c| c.file.as_ref()) {
        // Ensure the directory exists
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file_appender = RollingFileAppender::new(
            Rotation::NEVER,
            file_path.parent().unwrap_or_else(|| Path::new(".")),
            file_path.file_name().unwrap().to_str().unwrap(),
        );

        if format_str == "json" {
            let subscriber = subscriber.json().with_writer(file_appender);
            tracing::subscriber::set_global_default(subscriber.finish())?;
        } else {
            let subscriber = subscriber.with_writer(file_appender);
            tracing::subscriber::set_global_default(subscriber.finish())?;
        }

        tracing::info!("Logging to file: {}", file_path.display());
    } else {
        // Log to stdout
        if format_str == "json" {
            let subscriber = subscriber.json();
            tracing::subscriber::set_global_default(subscriber.finish())?;
        } else {
            tracing::subscriber::set_global_default(subscriber.finish())?;
        }
    }

    tracing::info!("Logging initialized with level: {}, format: {}", level_str, format_str);
    Ok(())
}
