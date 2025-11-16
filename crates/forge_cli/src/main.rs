//! # Forge CLI
//!
//! Command-line interface for the Forge local LLM platform.

use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "forge")]
#[command(about = "Forge - Local LLM Platform", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Forge daemon
    Daemon {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },

    /// List available models
    Models,

    /// List active sessions
    Sessions,

    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    match cli.command {
        Commands::Daemon { port } => {
            info!("Starting Forge daemon on port {}", port);
            forge_api::serve(port).await?;
        }
        Commands::Models => {
            info!("Listing models");
            println!("No models installed yet.");
        }
        Commands::Sessions => {
            info!("Listing sessions");
            println!("No active sessions.");
        }
        Commands::Version => {
            println!("Forge CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Copyright (c) 2025 Forge Contributors");
        }
    }

    Ok(())
}
