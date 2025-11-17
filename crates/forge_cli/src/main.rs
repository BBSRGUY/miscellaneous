//! # Forge CLI
//!
//! Command-line interface for the Forge local LLM platform.

use clap::{Parser, Subcommand};
use forge_runtime::config::AppConfig;
use forge_runtime::logging;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use tracing::{error, info};

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

    /// API base URL (default: http://localhost:3000)
    #[arg(long, global = true, default_value = "http://localhost:3000")]
    api_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Forge daemon (HTTP API server)
    Serve {
        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to (overrides config)
        #[arg(long)]
        host: Option<String>,
    },

    /// Interactive chat with a model
    Chat {
        /// Model ID to use for chat
        #[arg(short, long)]
        model: String,

        /// Enable streaming responses
        #[arg(short, long)]
        stream: bool,
    },

    /// Manage models
    #[command(subcommand)]
    Models(ModelsCommands),

    /// Manage jobs
    #[command(subcommand)]
    Jobs(JobsCommands),

    /// Show version information
    Version,

    /// Show current configuration
    Config {
        /// Show configuration as JSON
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ModelsCommands {
    /// List all registered models
    List,

    /// Register a new model
    Add {
        /// Model name
        #[arg(long)]
        name: String,

        /// Path to model file
        #[arg(long)]
        path: String,

        /// Backend type (e.g., "echo", "llama", "candle")
        #[arg(long, default_value = "echo")]
        backend: String,

        /// Model format (e.g., "gguf", "safetensors")
        #[arg(long, default_value = "gguf")]
        format: String,

        /// Quantization type (e.g., "q4_k_m", "q8_0")
        #[arg(long)]
        quantization: Option<String>,
    },

    /// Load a model into memory
    Load {
        /// Model ID to load
        id: String,

        /// Device to load on (cpu, gpu:0, gpu:1, etc.)
        #[arg(long, default_value = "cpu")]
        device: String,
    },
}

#[derive(Subcommand)]
enum JobsCommands {
    /// List all active jobs
    List,

    /// Get status of a specific job
    Status {
        /// Job ID
        id: String,
    },
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct ModelListItem {
    id: String,
    name: String,
    backend: String,
    format: String,
    quantization: String,
    size_bytes: Option<u64>,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct RegisterModelDto {
    name: String,
    path: String,
    backend: String,
    format: String,
    quantization: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegisterModelResponse {
    id: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ChatRequestDto {
    model_id: String,
    prompt: String,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    task_id: String,
    #[allow(dead_code)]
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct JobListItem {
    id: String,
    state: String,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct SseChunk {
    text: Option<String>,
    finish_reason: Option<String>,
    error: Option<String>,
}

// ============================================================================
// API Client Functions
// ============================================================================

async fn check_api_health(api_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/health", api_url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("API health check failed: {}", response.status());
    }

    let health: HealthResponse = response.json().await?;
    println!("✓ Connected to Forge API");
    println!("  Status: {}", health.status);
    println!("  Version: {}", health.version);

    Ok(())
}

async fn list_models(api_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/models", api_url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to list models: {}", response.status());
    }

    let models: Vec<ModelListItem> = response.json().await?;

    if models.is_empty() {
        println!("No models registered.");
        println!("\nTo add a model, use:");
        println!("  forge models add --name <name> --path <path> --backend <backend>");
        return Ok(());
    }

    println!("Registered Models:");
    println!("{:<20} {:<36} {:<10} {:<15} {:<10} {:<12}",
             "Name", "ID", "Backend", "Format", "Quant", "Size");
    println!("{}", "─".repeat(110));

    for model in models {
        let size = model.size_bytes
            .map(|s| format_size(s))
            .unwrap_or_else(|| "N/A".to_string());
        let status = if model.enabled { "" } else { " (disabled)" };

        println!("{:<20} {:<36} {:<10} {:<15} {:<10} {:<12}{}",
                 model.name, model.id, model.backend, model.format,
                 model.quantization, size, status);
    }

    Ok(())
}

async fn register_model(
    api_url: &str,
    name: String,
    path: String,
    backend: String,
    format: String,
    quantization: Option<String>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/models", api_url);

    let dto = RegisterModelDto {
        name: name.clone(),
        path: path.clone(),
        backend,
        format,
        quantization,
    };

    println!("Registering model '{}' from '{}'...", name, path);

    let response = client.post(&url).json(&dto).send().await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to register model: {}", error_text);
    }

    let result: RegisterModelResponse = response.json().await?;

    println!("✓ {}", result.message);
    println!("  Model ID: {}", result.id);

    Ok(())
}

async fn load_model(api_url: &str, model_id: String, device: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/models/{}/load", api_url, model_id);

    println!("Loading model '{}' on device '{}'...", model_id, device);

    let response = client
        .post(&url)
        .json(&serde_json::json!({ "device": device }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to load model: {}", error_text);
    }

    println!("✓ Model loaded successfully");

    Ok(())
}

async fn list_jobs(api_url: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/jobs", api_url);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to list jobs: {}", response.status());
    }

    let jobs: Vec<JobListItem> = response.json().await?;

    if jobs.is_empty() {
        println!("No active jobs.");
        return Ok(());
    }

    println!("Active Jobs:");
    println!("{:<36} {:<12} {:<20} {:<12}", "Job ID", "State", "Created", "Duration");
    println!("{}", "─".repeat(85));

    for job in jobs {
        let duration = job.duration_ms
            .map(|ms| format!("{}ms", ms))
            .unwrap_or_else(|| "-".to_string());

        println!("{:<36} {:<12} {:<20} {:<12}",
                 job.id, job.state, format_timestamp(&job.created_at), duration);
    }

    Ok(())
}

async fn get_job_status(api_url: &str, job_id: String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/jobs/{}", api_url, job_id);

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to get job status: {}", response.status());
    }

    let job: JobListItem = response.json().await?;

    println!("Job Status:");
    println!("  ID: {}", job.id);
    println!("  State: {}", job.state);
    println!("  Created: {}", job.created_at);

    if let Some(started) = job.started_at {
        println!("  Started: {}", started);
    }

    if let Some(completed) = job.completed_at {
        println!("  Completed: {}", completed);
    }

    if let Some(duration) = job.duration_ms {
        println!("  Duration: {}ms", duration);
    }

    Ok(())
}

async fn interactive_chat(api_url: &str, model_id: String, stream: bool) -> anyhow::Result<()> {
    println!("Forge Chat - Model: {}", model_id);
    println!("Type 'exit' or 'quit' to end the session.\n");

    // Check if API is available
    check_api_health(api_url).await?;

    let client = reqwest::Client::new();

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let prompt = input.trim();

        if prompt.is_empty() {
            continue;
        }

        if prompt == "exit" || prompt == "quit" {
            println!("Goodbye!");
            break;
        }

        print!("Assistant: ");
        io::stdout().flush()?;

        if stream {
            // Streaming response using SSE
            let url = format!("{}/api/v1/chat", api_url);
            let dto = ChatRequestDto {
                model_id: model_id.clone(),
                prompt: prompt.to_string(),
                stream: Some(true),
            };

            let response = client.post(&url).json(&dto).send().await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                println!("\nError: {}", error_text);
                continue;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process complete SSE events
                        while let Some(event_end) = buffer.find("\n\n") {
                            let event = buffer[..event_end].to_string();
                            buffer = buffer[event_end + 2..].to_string();

                            // Parse SSE event
                            for line in event.lines() {
                                if let Some(data) = line.strip_prefix("data:") {
                                    if let Ok(chunk) = serde_json::from_str::<SseChunk>(data.trim()) {
                                        if let Some(error) = chunk.error {
                                            println!("\nError: {}", error);
                                            break;
                                        }
                                        if let Some(text) = chunk.text {
                                            print!("{}", text);
                                            io::stdout().flush()?;
                                        }
                                        if chunk.finish_reason.is_some() {
                                            println!();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("\nStream error: {}", e);
                        break;
                    }
                }
            }
            println!();
        } else {
            // Non-streaming response
            let url = format!("{}/api/v1/chat", api_url);
            let dto = ChatRequestDto {
                model_id: model_id.clone(),
                prompt: prompt.to_string(),
                stream: Some(false),
            };

            let response = client.post(&url).json(&dto).send().await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                println!("\nError: {}", error_text);
                continue;
            }

            let result: ChatResponse = response.json().await?;
            println!("Task submitted: {}", result.task_id);
            println!("(Use 'forge jobs status {}' to check status)", result.task_id);
            println!();
        }
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_timestamp(timestamp: &str) -> String {
    // Just take the first 19 characters (YYYY-MM-DDTHH:MM:SS)
    timestamp.chars().take(19).collect()
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port, host } => {
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

        Commands::Chat { model, stream } => {
            interactive_chat(&cli.api_url, model, stream).await?;
        }

        Commands::Models(models_cmd) => {
            match models_cmd {
                ModelsCommands::List => {
                    list_models(&cli.api_url).await?;
                }
                ModelsCommands::Add { name, path, backend, format, quantization } => {
                    register_model(&cli.api_url, name, path, backend, format, quantization).await?;
                }
                ModelsCommands::Load { id, device } => {
                    load_model(&cli.api_url, id, device).await?;
                }
            }
        }

        Commands::Jobs(jobs_cmd) => {
            match jobs_cmd {
                JobsCommands::List => {
                    list_jobs(&cli.api_url).await?;
                }
                JobsCommands::Status { id } => {
                    get_job_status(&cli.api_url, id).await?;
                }
            }
        }

        Commands::Version => {
            println!("Forge CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("Copyright (c) 2025 Forge Contributors");
            println!();
            println!("Build information:");
            println!("  Rust version: {}", env!("CARGO_PKG_RUST_VERSION"));
        }

        Commands::Config { json } => {
            let config = match AppConfig::load_from(&cli.config) {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to load configuration: {}", e);
                    eprintln!("Failed to load configuration: {}", e);
                    std::process::exit(1);
                }
            };

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
