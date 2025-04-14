mod config;
mod app;

use anyhow::Result;
use clap::Parser;
use config::Config;
use std::path::PathBuf;
use crate::app::server::Server;
use crate::config::ServerConfig;

/// Git-Actions: A Rust-based automation tool for Git events
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the server configuration file
    #[clap(short, long, default_value = "server.yaml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Load configuration
    // let config = Config::load(&args.config)?;
    let config = ServerConfig::from_file(&args.config)?;

    // run the server
    app::run(config).await?;

    Ok(())
}
