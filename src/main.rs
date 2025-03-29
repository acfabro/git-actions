mod config;
mod logging;
mod server;

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
    let config = ServerConfig::from_file(&args.config)?;

    // run the server
    server::run(config).await?;

    Ok(())
}
