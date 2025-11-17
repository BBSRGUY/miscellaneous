//! # Forge API
//!
//! HTTP/JSON API for the Forge platform using Axum.
//!
//! This crate provides a versioned REST API layer exposing endpoints for
//! health checks, model management, chat completions with streaming, session
//! management, and job tracking.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use forge_engine::{DeviceKind, InferenceRequest};
use forge_models::{Format, Quantization, RegisterModelRequest};
use forge_runtime::{ChatRequest, Runtime, TaskPriority};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tower_http::cors::CorsLayer;
use tracing::{debug, info};

pub use forge_engine;
pub use forge_models;
pub use forge_runtime;
pub use forge_store;

/// API version prefix
pub const API_VERSION: &str = "v1";

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

    #[error("Session error: {0}")]
    Session(#[from] forge_runtime::SessionError),

    #[error("Scheduler error: {0}")]
    Scheduler(#[from] forge_runtime::SchedulerError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
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
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;

// ============================================================================
// DTOs - Request/Response Types
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Configuration response (sanitized).
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub api_port: u16,
    pub models_dir: String,
    pub storage_dir: String,
    pub log_level: String,
}

/// Model list item.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelListItem {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub format: String,
    pub quantization: String,
    pub size_bytes: Option<u64>,
    pub enabled: bool,
}

/// Register model request.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterModelDto {
    pub name: String,
    pub path: String,
    pub backend: String,
    pub format: String,
    pub quantization: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Load model request.
#[derive(Debug, Deserialize)]
pub struct LoadModelRequest {
    pub device: Option<String>,
}

/// Chat request.
#[derive(Debug, Deserialize)]
pub struct ChatRequestDto {
    pub session_id: Option<String>,
    pub model_id: String,
    pub prompt: String,
    pub stream: Option<bool>,
    pub priority: Option<String>,
}

/// Chat response (non-streaming).
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub task_id: String,
    pub session_id: String,
}

/// Session detail response.
#[derive(Debug, Serialize)]
pub struct SessionDetail {
    pub id: String,
    pub name: String,
    pub model_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub messages: Vec<MessageDto>,
}

/// Message DTO.
#[derive(Debug, Serialize)]
pub struct MessageDto {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// Job list item.
#[derive(Debug, Serialize, Deserialize)]
pub struct JobListItem {
    pub id: String,
    pub state: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check handler.
#[tracing::instrument]
async fn health_handler() -> Json<HealthResponse> {
    debug!("Health check requested");
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0,
    })
}

/// Configuration handler (sanitized).
#[tracing::instrument(skip(_runtime))]
async fn config_handler(State(_runtime): State<Arc<Runtime>>) -> Json<ConfigResponse> {
    debug!("Configuration requested");
    Json(ConfigResponse {
        api_port: 3000,
        models_dir: "/data/models".to_string(),
        storage_dir: "/data/storage".to_string(),
        log_level: "info".to_string(),
    })
}

/// List models handler.
#[tracing::instrument(skip(runtime))]
async fn list_models_handler(
    State(runtime): State<Arc<Runtime>>,
) -> ApiResult<Json<Vec<ModelListItem>>> {
    debug!("Listing models");
    let models = runtime.models().list_models().await?;

    let items: Vec<ModelListItem> = models
        .into_iter()
        .map(|m| ModelListItem {
            id: m.id,
            name: m.name,
            backend: m.backend,
            format: m.format.to_string(),
            quantization: m.quantization.to_string(),
            size_bytes: m.size_bytes,
            enabled: m.enabled,
        })
        .collect();

    Ok(Json(items))
}

/// Register model handler.
#[tracing::instrument(skip(runtime))]
async fn register_model_handler(
    State(runtime): State<Arc<Runtime>>,
    Json(dto): Json<RegisterModelDto>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Registering model: {}", dto.name);

    let format = dto.format.parse::<Format>()
        .map_err(|e| ApiError::BadRequest(e))?;

    let quantization = dto.quantization
        .as_ref()
        .and_then(|q| q.parse::<Quantization>().ok());

    let request = RegisterModelRequest {
        name: dto.name,
        path: PathBuf::from(dto.path),
        backend: dto.backend,
        format,
        quantization,
        tags: dto.tags.unwrap_or_default(),
    };

    let model_id = runtime.register_model(request).await?;

    Ok(Json(serde_json::json!({
        "id": model_id,
        "message": "Model registered successfully"
    })))
}

/// Load model handler.
#[tracing::instrument(skip(runtime))]
async fn load_model_handler(
    State(runtime): State<Arc<Runtime>>,
    Path(model_id): Path<String>,
    Json(req): Json<LoadModelRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Loading model: {}", model_id);

    let device = match req.device.as_deref() {
        Some("cpu") | None => DeviceKind::CPU,
        Some(d) if d.starts_with("gpu:") => {
            let id = d.strip_prefix("gpu:")
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| ApiError::BadRequest("Invalid GPU ID".to_string()))?;
            DeviceKind::GPU { id }
        }
        Some(d) => return Err(ApiError::BadRequest(format!("Unknown device: {}", d))),
    };

    runtime.models().ensure_model_loaded(&model_id, device).await?;

    Ok(Json(serde_json::json!({
        "message": "Model loaded successfully"
    })))
}

/// Chat handler with optional streaming.
#[tracing::instrument(skip(runtime))]
async fn chat_handler(
    State(runtime): State<Arc<Runtime>>,
    Json(dto): Json<ChatRequestDto>,
) -> ApiResult<Response> {
    info!("Chat request for model: {}", dto.model_id);

    let priority = dto.priority
        .as_deref()
        .and_then(|p| match p.to_lowercase().as_str() {
            "low" => Some(TaskPriority::Low),
            "normal" => Some(TaskPriority::Normal),
            "high" => Some(TaskPriority::High),
            "critical" => Some(TaskPriority::Critical),
            _ => None,
        });

    if dto.stream.unwrap_or(false) {
        let request = InferenceRequest::from_prompt(
            dto.model_id.clone(),
            dto.prompt.clone(),
        );

        runtime.models()
            .ensure_model_loaded(&dto.model_id, DeviceKind::CPU)
            .await?;

        let stream = runtime.run_inference(&dto.model_id, request).await?;

        let sse_stream = stream.map(|result| match result {
            Ok(chunk) => {
                let data = serde_json::json!({
                    "text": chunk.text,
                    "finish_reason": chunk.finish_reason,
                });
                Ok::<_, Infallible>(axum::response::sse::Event::default()
                    .json_data(data)
                    .unwrap())
            }
            Err(e) => {
                let error_data = serde_json::json!({
                    "error": e.to_string(),
                });
                Ok(axum::response::sse::Event::default()
                    .json_data(error_data)
                    .unwrap())
            }
        });

        Ok(Sse::new(sse_stream).into_response())
    } else {
        let chat_request = ChatRequest {
            session_id: dto.session_id,
            model_id: dto.model_id,
            prompt: dto.prompt,
            priority,
        };

        let task_id = runtime.start_chat(chat_request).await?;

        let response = ChatResponse {
            task_id: task_id.clone(),
            session_id: "session-placeholder".to_string(),
        };

        Ok(Json(response).into_response())
    }
}

/// Get session details handler.
#[tracing::instrument(skip(runtime))]
async fn get_session_handler(
    State(runtime): State<Arc<Runtime>>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<SessionDetail>> {
    debug!("Getting session: {}", session_id);

    let session = runtime.sessions().get_session(&session_id).await?;

    let messages: Vec<MessageDto> = session
        .messages
        .into_iter()
        .map(|m| MessageDto {
            id: m.id,
            role: match m.role {
                forge_runtime::MessageRole::User => "user".to_string(),
                forge_runtime::MessageRole::Assistant => "assistant".to_string(),
                forge_runtime::MessageRole::System => "system".to_string(),
            },
            content: m.content,
            created_at: m.created_at.to_rfc3339(),
        })
        .collect();

    let detail = SessionDetail {
        id: session.id,
        name: session.name,
        model_id: session.model_id,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        message_count: messages.len(),
        messages,
    };

    Ok(Json(detail))
}

/// List jobs handler.
#[tracing::instrument(skip(runtime))]
async fn list_jobs_handler(
    State(runtime): State<Arc<Runtime>>,
) -> Json<Vec<JobListItem>> {
    debug!("Listing jobs");

    let jobs = runtime.get_active_jobs();

    let items: Vec<JobListItem> = jobs
        .into_iter()
        .map(|j| JobListItem {
            id: j.id,
            state: format!("{:?}", j.state),
            created_at: j.created_at.to_rfc3339(),
            started_at: j.started_at.map(|t| t.to_rfc3339()),
            completed_at: j.completed_at.map(|t| t.to_rfc3339()),
            duration_ms: j.duration_ms,
        })
        .collect();

    Json(items)
}

/// Get job details handler.
#[tracing::instrument(skip(runtime))]
async fn get_job_handler(
    State(runtime): State<Arc<Runtime>>,
    Path(job_id): Path<String>,
) -> ApiResult<Json<JobListItem>> {
    debug!("Getting job: {}", job_id);

    let job = runtime.get_job_status(&job_id).await?;

    let item = JobListItem {
        id: job.id,
        state: format!("{:?}", job.state),
        created_at: job.created_at.to_rfc3339(),
        started_at: job.started_at.map(|t| t.to_rfc3339()),
        completed_at: job.completed_at.map(|t| t.to_rfc3339()),
        duration_ms: job.duration_ms,
    };

    Ok(Json(item))
}

// ============================================================================
// Router Setup
// ============================================================================

/// Create the versioned API router.
pub fn create_api_router(runtime: Arc<Runtime>) -> Router {
    let v1_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/config", get(config_handler))
        .route("/models", get(list_models_handler).post(register_model_handler))
        .route("/models/:id/load", post(load_model_handler))
        .route("/chat", post(chat_handler))
        .route("/sessions/:id", get(get_session_handler))
        .route("/jobs", get(list_jobs_handler))
        .route("/jobs/:id", get(get_job_handler))
        .with_state(runtime);

    Router::new()
        .nest(&format!("/api/{}", API_VERSION), v1_routes)
        .layer(CorsLayer::permissive())
}

/// Run the HTTP server.
pub async fn run_http_server(
    runtime: Arc<Runtime>,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    info!("Starting Forge HTTP API server");
    info!("API Version: {}", API_VERSION);

    let app = create_api_router(runtime);

    let addr = format!("{}:{}", host, port);
    info!("Binding to address: {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("API server started successfully");
    info!("Listening on http://{}", addr);
    info!("Available endpoints:");
    info!("  GET  /api/{}/health", API_VERSION);
    info!("  GET  /api/{}/config", API_VERSION);
    info!("  GET  /api/{}/models", API_VERSION);
    info!("  POST /api/{}/models", API_VERSION);
    info!("  POST /api/{}/models/{{id}}/load", API_VERSION);
    info!("  POST /api/{}/chat", API_VERSION);
    info!("  GET  /api/{}/sessions/{{id}}", API_VERSION);
    info!("  GET  /api/{}/jobs", API_VERSION);
    info!("  GET  /api/{}/jobs/{{id}}", API_VERSION);

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::EchoBackend;
    use forge_store::Store;
    use tempfile::TempDir;

    async fn setup_test_runtime() -> (Arc<Runtime>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        let backend = Arc::new(EchoBackend::new());
        let engine = Arc::new(forge_engine::Engine::new(backend));
        let models = Arc::new(forge_models::ModelRegistry::with_engine(
            store.clone(),
            engine.clone(),
        ));

        let runtime = Arc::new(Runtime::new(store, models, engine));

        (runtime, temp_dir)
    }

    fn create_temp_model_file(temp_dir: &TempDir, name: &str) -> PathBuf {
        let path = temp_dir.path().join(name);
        std::fs::write(&path, b"fake model data").unwrap();
        path
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let (runtime, _temp_dir) = setup_test_runtime().await;

        let app = create_api_router(runtime);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/v1/health", addr);
        let response = client.get(&url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: HealthResponse = response.json().await.unwrap();
        assert_eq!(body.status, "healthy");
    }

    #[tokio::test]
    async fn test_list_models_empty() {
        let (runtime, _temp_dir) = setup_test_runtime().await;

        let app = create_api_router(runtime);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/v1/models", addr);
        let response = client.get(&url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let models: Vec<ModelListItem> = response.json().await.unwrap();
        assert_eq!(models.len(), 0);
    }

    #[tokio::test]
    async fn test_register_and_list_model() {
        let (runtime, temp_dir) = setup_test_runtime().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let app = create_api_router(runtime);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        let register_dto = RegisterModelDto {
            name: "test-model".to_string(),
            path: model_path.display().to_string(),
            backend: "echo".to_string(),
            format: "gguf".to_string(),
            quantization: Some("q4_k_m".to_string()),
            tags: Some(vec!["test".to_string()]),
        };

        let url = format!("http://{}/api/v1/models", addr);
        let response = client
            .post(&url)
            .json(&register_dto)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let url = format!("http://{}/api/v1/models", addr);
        let response = client.get(&url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let models: Vec<ModelListItem> = response.json().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test-model");
        assert_eq!(models[0].backend, "echo");
    }

    #[tokio::test]
    async fn test_jobs_list() {
        let (runtime, _temp_dir) = setup_test_runtime().await;

        let app = create_api_router(runtime);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/v1/jobs", addr);
        let response = client.get(&url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let jobs: Vec<JobListItem> = response.json().await.unwrap();
        assert_eq!(jobs.len(), 0);
    }
}
