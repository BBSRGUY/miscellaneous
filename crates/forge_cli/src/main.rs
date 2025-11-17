//! # Forge CLI
//!
//! Command-line interface for the Forge local LLM platform.

use clap::{Parser, Subcommand};
use forge_runtime::config::AppConfig;
use forge_runtime::logging;
use tracing::info;

#[derive(Parser)]
#[command(name = "forge")]
#[command(about = "Forge - Local LLM Platform", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging (overrides config)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging (overrides config)
    #[arg(short, long, global = true)]
    debug: bool,

    /// Config directory path
    #[arg(short, long, global = true, default_value = "configs")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Forge daemon
    Daemon {
        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to (overrides config)
        #[arg(long)]
        host: Option<String>,
    },

    /// List available models
    Models,

    /// List active sessions
    Sessions,

    /// Show version information
    Version,

    /// Show current configuration
    Config {
        /// Show configuration as JSON
        #[arg(short, long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut config = match AppConfig::load_from(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Make sure configs/default.yaml exists");
            std::process::exit(1);
        }
    };

    // Override log level if verbose or debug flags are set
    if cli.debug {
        config.logging.level = "debug".to_string();
    } else if cli.verbose {
        config.logging.level = "trace".to_string();
    }

    // Initialize logging
    if let Err(e) = logging::init_logging(&config.logging) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Forge CLI v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from: {}", cli.config);

    match cli.command {
        Commands::Daemon { port, host } => {
            // Override config with CLI arguments if provided
            if let Some(p) = port {
                info!("Overriding API port from config: {} -> {}", config.api.port, p);
                config.api.port = p;
            }
            if let Some(h) = host {
                info!("Overriding API host from config: {} -> {}", config.api.host, h);
                config.api.host = h;
            }

            info!("Starting Forge daemon");
            info!("API server will listen on {}:{}", config.api.host, config.api.port);
            info!("Database path: {}", config.db_path().display());
            info!("Models directory: {}", config.models_path().display());

            // Initialize runtime components
            use std::sync::Arc;
            use forge_store::Store;
            use forge_engine::Engine;
            use forge_models::ModelRegistry;
            use forge_runtime::Runtime;

            let db_path = config.db_path();
            let blob_storage = config.storage.data_dir.join("blobs");
            let store = Arc::new(Store::new_with_file(&db_path, &blob_storage).await?);

            // TODO: Initialize proper backend based on config
            // For now, use echo backend as placeholder
            let backend = Arc::new(forge_engine::EchoBackend::new());
            let engine = Arc::new(Engine::new(backend));
            let models = Arc::new(ModelRegistry::with_engine(store.clone(), engine.clone()));
            let runtime = Arc::new(Runtime::new(store, models, engine));

            forge_api::run_http_server(runtime, &config.api.host, config.api.port).await?;
        }
        Commands::Models => {
            info!("Listing models from: {}", config.models_path().display());
            println!("Models directory: {}", config.models_path().display());
            println!("No models installed yet.");
            println!("\nTo add models, place them in the models directory.");
        }
        Commands::Sessions => {
            info!("Listing sessions");
            println!("Database: {}", config.db_path().display());
            println!("No active sessions.");
        }
        Commands::Version => {
            println!("Forge CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Copyright (c) 2025 Forge Contributors");
            println!();
            println!("Build information:");
            println!("  Rust version: {}", env!("CARGO_PKG_RUST_VERSION"));
            println!("  Config directory: {}", cli.config);
        }
        Commands::Config { json } => {
            info!("Displaying configuration");
            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("Configuration:");
                println!("  API:");
                println!("    Host: {}", config.api.host);
                println!("    Port: {}", config.api.port);
                println!("    CORS: {}", config.api.cors_enabled);
                println!();
                println!("  Database:");
                println!("    Path: {}", config.db_path().display());
                println!("    WAL mode: {}", config.db.wal_mode);
                println!("    Pool size: {}", config.db.pool_size);
                println!();
                println!("  Storage:");
                println!("    Data directory: {}", config.storage.data_dir.display());
                println!("    Models: {}", config.models_path().display());
                println!("    Datasets: {}", config.datasets_path().display());
                println!();
                println!("  Logging:");
                println!("    Level: {}", config.logging.level);
                println!("    Format: {:?}", config.logging.format);
                println!();
                println!("  Inference:");
                println!("    Max concurrent tasks: {}", config.inference.max_concurrent_tasks);
                println!("    Default temperature: {}", config.inference.default_temperature);
                println!("    Default max tokens: {}", config.inference.default_max_tokens);
                println!();
                println!("  GPU:");
                println!("    Enabled: {}", config.gpu.enabled);
                println!("    High performance: {}", config.gpu.high_performance);
            }
        }
    }

    Ok(())
}
