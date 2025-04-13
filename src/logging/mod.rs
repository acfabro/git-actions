use crate::config::server_config::LoggingSpec;
use anyhow::Result;
use std::path::Path;
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::SubscriberBuilder;

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

    info!("Logging level: {}", level_str);
    info!("Logging format: {}", format_str);

    // Set up the subscriber
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&level_str));

    // Configure the subscriber based on the format
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter);

    // If a log file is specified, use it
    if let Some(file_path) = config.as_ref().and_then(|c| c.file.as_ref()) {
        // TODO implement logging by file at path

        info!("Logging to file: {}", file_path.display());

    } else {
        // Log to stdout
        if format_str == "json" {
            let subscriber = subscriber.json();
            tracing::subscriber::set_global_default(subscriber.finish())?;
        } else {
            tracing::subscriber::set_global_default(subscriber.finish())?;
        }
    }

    info!("Logging initialized with level: {}, format: {}", level_str, format_str);
    Ok(())
}
