mod config;
mod logging;
mod server;

use anyhow::Context;
use clap::Parser;
use config::ServerConfig;
use std::path::PathBuf;

/// Git-Actions: A Rust-based automation tool for Git events
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the server configuration file
    #[clap(short, long, value_parser, default_value = "server.yaml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Load server configuration
    let config_path = args.config;
    let config = ServerConfig::from_file(&config_path)?;

    // Set up logging based on configuration
    logging::setup(&config.spec.logging)
        .with_context(|| format!("Failed to setup logging for {}", config_path.display()))?;

    server::run(config).await?;

    Ok(())
}
