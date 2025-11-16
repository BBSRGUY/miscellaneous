//! # Forge API
//!
//! HTTP/JSON API for the Forge platform.
//!
//! This crate provides the REST API layer using Axum, exposing endpoints
//! for health checks, model management, chat completions, and more.

use anyhow;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tower_http::cors::CorsLayer;
use tracing::info;

pub use forge_engine;
pub use forge_models;
pub use forge_runtime;
pub use forge_store;

/// API-level errors.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Model error: {0}")]
    Model(#[from] forge_models::ModelError),

    #[error("Runtime error: {0}")]
    Runtime(#[from] forge_runtime::RuntimeError),

    #[error("Store error: {0}")]
    Store(#[from] forge_store::StoreError),

    #[error("Engine error: {0}")]
    Engine(#[from] forge_engine::EngineError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub model_registry: Arc<forge_models::ModelRegistry>,
    pub session_manager: Arc<forge_runtime::SessionManager>,
    pub task_scheduler: Arc<forge_runtime::TaskScheduler>,
}

impl AppState {
    /// Create new application state.
    pub fn new() -> Self {
        Self {
            model_registry: Arc::new(forge_models::ModelRegistry::new()),
            session_manager: Arc::new(forge_runtime::SessionManager::new()),
            task_scheduler: Arc::new(forge_runtime::TaskScheduler::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check handler.
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Status response.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub models_loaded: usize,
    pub active_sessions: usize,
    pub queued_tasks: usize,
}

/// Status handler.
async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let models = state.model_registry.list();
    let sessions = state.session_manager.list_sessions();

    Json(StatusResponse {
        models_loaded: models.len(),
        active_sessions: sessions.len(),
        queued_tasks: 0, // TODO: implement queue size tracking
    })
}

/// List models handler.
async fn list_models_handler(
    State(state): State<AppState>,
) -> Json<Vec<forge_models::ModelEntry>> {
    let models = state.model_registry.list();
    Json(models)
}

/// List sessions handler.
async fn list_sessions_handler(
    State(state): State<AppState>,
) -> Json<Vec<forge_runtime::Session>> {
    let sessions = state.session_manager.list_sessions();
    Json(sessions)
}

/// Create the API router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/models", get(list_models_handler))
        .route("/sessions", get(list_sessions_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the API server.
pub async fn serve(port: u16) -> anyhow::Result<()> {
    let state = AppState::new();
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!("API server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert_eq!(state.model_registry.list().len(), 0);
        assert_eq!(state.session_manager.list_sessions().len(), 0);
    }
}
