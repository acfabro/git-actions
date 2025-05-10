use super::config::server::LoggingSpec;
use anyhow::Result;
use tracing::info;
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
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&level_str));

    // Configure the subscriber based on the format
    let subscriber = fmt::Subscriber::builder().with_env_filter(env_filter);

    // Log to stdout
    if format_str == "json" {
        let subscriber = subscriber.json();
        tracing::subscriber::set_global_default(subscriber.finish())?;
    } else {
        tracing::subscriber::set_global_default(subscriber.finish())?;
    }

    info!(
        "Logging initialized with level: {}, format: {}",
        level_str, format_str
    );
    Ok(())
}
