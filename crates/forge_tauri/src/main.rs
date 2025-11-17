//! # Forge Tauri Desktop App
//!
//! Desktop shell for the Forge platform using Tauri with embedded terminal.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

/// Shared application state for Tauri commands.
struct AppState {
    daemon_process: Arc<Mutex<Option<Child>>>,
    terminal_process: Arc<Mutex<Option<Child>>>,
    api_url: String,
}

/// Health check response from API.
#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Model list item from API.
#[derive(Debug, Serialize, Deserialize)]
struct ModelListItem {
    id: String,
    name: String,
    backend: String,
    format: String,
    quantization: String,
    size_bytes: Option<u64>,
    enabled: bool,
}

/// Job list item from API.
#[derive(Debug, Serialize, Deserialize)]
struct JobListItem {
    id: String,
    state: String,
    created_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    duration_ms: Option<u64>,
}

// ============================================================================
// Daemon Management Commands
// ============================================================================

/// Start the Forge daemon (HTTP API server).
#[tauri::command]
async fn start_daemon(state: State<'_, AppState>) -> Result<String, String> {
    info!("Starting Forge daemon");

    let mut daemon = state.daemon_process.lock().unwrap();

    if daemon.is_some() {
        return Ok("Daemon is already running".to_string());
    }

    // Spawn forge_cli serve process
    let child = Command::new("target/debug/forge")
        .args(&["serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn daemon: {}", e))?;

    *daemon = Some(child);

    // Wait for API to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    Ok("Daemon started successfully".to_string())
}

/// Check if the API is healthy and reachable.
#[tauri::command]
async fn check_api_health(state: State<'_, AppState>) -> Result<HealthResponse, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/health", state.api_url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to API: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API health check failed: {}", response.status()));
    }

    let health: HealthResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse health response: {}", e))?;

    Ok(health)
}

/// Stop the Forge daemon.
#[tauri::command]
async fn stop_daemon(state: State<'_, AppState>) -> Result<String, String> {
    info!("Stopping Forge daemon");

    let mut daemon = state.daemon_process.lock().unwrap();

    if let Some(mut child) = daemon.take() {
        child
            .kill()
            .map_err(|e| format!("Failed to kill daemon: {}", e))?;
        child
            .wait()
            .map_err(|e| format!("Failed to wait for daemon: {}", e))?;
        Ok("Daemon stopped successfully".to_string())
    } else {
        Ok("Daemon is not running".to_string())
    }
}

// ============================================================================
// API Interaction Commands
// ============================================================================

/// List all models via HTTP API.
#[tauri::command]
async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelListItem>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/models", state.api_url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to list models: {}", response.status()));
    }

    let models: Vec<ModelListItem> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse models: {}", e))?;

    Ok(models)
}

/// List all jobs via HTTP API.
#[tauri::command]
async fn list_jobs(state: State<'_, AppState>) -> Result<Vec<JobListItem>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/jobs", state.api_url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to list jobs: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to list jobs: {}", response.status()));
    }

    let jobs: Vec<JobListItem> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse jobs: {}", e))?;

    Ok(jobs)
}

// ============================================================================
// Terminal Commands
// ============================================================================

/// Spawn a forge CLI process for the embedded terminal.
#[tauri::command]
async fn spawn_terminal(state: State<'_, AppState>) -> Result<String, String> {
    info!("Spawning terminal process");

    let mut terminal = state.terminal_process.lock().unwrap();

    if terminal.is_some() {
        return Err("Terminal process is already running".to_string());
    }

    // For now, just spawn a shell
    // In a real implementation, you'd use a PTY library
    let child = Command::new("sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn terminal: {}", e))?;

    *terminal = Some(child);

    Ok("Terminal spawned successfully".to_string())
}

/// Write input to the terminal process.
#[tauri::command]
async fn terminal_input(_state: State<'_, AppState>, _input: String) -> Result<(), String> {
    // This would write to the PTY stdin
    // For now, it's a placeholder
    Ok(())
}

/// Resize the terminal.
#[tauri::command]
async fn terminal_resize(_state: State<'_, AppState>, _rows: u16, _cols: u16) -> Result<(), String> {
    // This would resize the PTY
    // For now, it's a placeholder
    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    // Set up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Starting Forge desktop application");

    // Initialize application state
    let app_state = AppState {
        daemon_process: Arc::new(Mutex::new(None)),
        terminal_process: Arc::new(Mutex::new(None)),
        api_url: "http://localhost:3000".to_string(),
    };

    tauri::Builder::default()
        .setup(|app| {
            // Start daemon on application startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("Attempting to start daemon on startup");
                match app_handle.state::<AppState>().daemon_process.lock() {
                    Ok(mut daemon) => {
                        if daemon.is_none() {
                            match Command::new("target/debug/forge")
                                .args(&["serve"])
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn()
                            {
                                Ok(child) => {
                                    *daemon = Some(child);
                                    info!("Daemon started successfully on startup");

                                    // Wait for daemon to be ready
                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                }
                                Err(e) => {
                                    error!("Failed to start daemon on startup: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to lock daemon process: {}", e);
                    }
                }
            });

            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_daemon,
            check_api_health,
            stop_daemon,
            list_models,
            list_jobs,
            spawn_terminal,
            terminal_input,
            terminal_resize,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
