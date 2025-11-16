//! # Forge Tauri Desktop App
//!
//! Desktop shell for the Forge platform using Tauri.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Shared application state for Tauri commands.
struct AppState {
    model_registry: Arc<forge_models::ModelRegistry>,
    session_manager: Arc<forge_runtime::SessionManager>,
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Tauri command: health check.
#[tauri::command]
fn health() -> HealthResponse {
    HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Tauri command: list models.
#[tauri::command]
fn list_models(state: State<AppState>) -> Vec<forge_models::ModelEntry> {
    state.model_registry.list()
}

/// Tauri command: list sessions.
#[tauri::command]
fn list_sessions(state: State<AppState>) -> Vec<forge_runtime::Session> {
    state.session_manager.list_sessions()
}

/// Tauri command: greet (example).
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Forge.", name)
}

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
        model_registry: Arc::new(forge_models::ModelRegistry::new()),
        session_manager: Arc::new(forge_runtime::SessionManager::new()),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            health,
            greet,
            list_models,
            list_sessions
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
