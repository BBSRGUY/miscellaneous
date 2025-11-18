//! # Runtime Context
//!
//! High-level runtime that coordinates store, models, engine, and scheduler.

use crate::scheduler::{Scheduler, SchedulerError};
use crate::sessions::{MessageRole, SessionManager};
use crate::tasks::{JobStatus, TaskPriority, TaskType};
use crate::training_jobs::TrainingJobManager;
use forge_engine::{Engine, InferenceRequest, InferenceStream, TrainBatch, TrainConfig};
use forge_models::{ModelRegistry, RegisterModelRequest};
use forge_store::Store;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

/// Errors that can occur during runtime operations.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Store error: {0}")]
    Store(#[from] forge_store::StoreError),

    #[error("Model error: {0}")]
    Model(#[from] forge_models::ModelError),

    #[error("Engine error: {0}")]
    Engine(#[from] forge_engine::EngineError),

    #[error("Scheduler error: {0}")]
    Scheduler(#[from] SchedulerError),

    #[error("Session error: {0}")]
    Session(#[from] crate::sessions::SessionError),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Request to start a chat/completion inference.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// Session ID (optional, creates new if not provided)
    pub session_id: Option<String>,
    /// Model ID to use
    pub model_id: String,
    /// User prompt
    pub prompt: String,
    /// Priority (defaults to High for interactive)
    pub priority: Option<TaskPriority>,
}

/// High-level runtime for coordinating all Forge components.
pub struct Runtime {
    store: Arc<Store>,
    models: Arc<ModelRegistry>,
    engine: Arc<Engine>,
    sessions: Arc<SessionManager>,
    scheduler: Arc<Scheduler>,
    training_jobs: Arc<TrainingJobManager>,
}

impl Runtime {
    /// Create a new runtime with all components.
    pub fn new(
        store: Arc<Store>,
        models: Arc<ModelRegistry>,
        engine: Arc<Engine>,
    ) -> Self {
        let sessions = Arc::new(SessionManager::new(store.clone()));
        let scheduler = Arc::new(Scheduler::new());
        let training_jobs = Arc::new(TrainingJobManager::new());

        Self {
            store,
            models,
            engine,
            sessions,
            scheduler,
            training_jobs,
        }
    }

    /// Get the store reference.
    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    /// Get the model registry reference.
    pub fn models(&self) -> &Arc<ModelRegistry> {
        &self.models
    }

    /// Get the engine reference.
    pub fn engine(&self) -> &Arc<Engine> {
        &self.engine
    }

    /// Get the session manager reference.
    pub fn sessions(&self) -> &Arc<SessionManager> {
        &self.sessions
    }

    /// Get the scheduler reference.
    pub fn scheduler(&self) -> &Arc<Scheduler> {
        &self.scheduler
    }

    /// Start a chat/completion task.
    ///
    /// This is a high-level method that:
    /// 1. Creates or uses existing session
    /// 2. Adds user message to session
    /// 3. Ensures model is loaded
    /// 4. Submits inference task
    /// 5. Returns task ID for tracking
    pub async fn start_chat(&self, request: ChatRequest) -> Result<String> {
        info!("Starting chat with model: {}", request.model_id);

        // Get or create session
        let session_id = if let Some(sid) = request.session_id {
            sid
        } else {
            let session = self.sessions
                .create_session("Chat Session".to_string(), Some(request.model_id.clone()))
                .await?;
            session.id
        };

        debug!("Using session: {}", session_id);

        // Add user message to session
        self.sessions
            .add_message(&session_id, MessageRole::User, request.prompt.clone())
            .await?;

        // Get message history for context
        let messages = self.sessions.get_messages(&session_id).await?;

        // Convert to engine messages
        let engine_messages: Vec<forge_engine::Message> = messages
            .into_iter()
            .map(|m| forge_engine::Message {
                role: match m.role {
                    MessageRole::User => forge_engine::MessageRole::User,
                    MessageRole::Assistant => forge_engine::MessageRole::Assistant,
                    MessageRole::System => forge_engine::MessageRole::System,
                },
                content: m.content,
            })
            .collect();

        // Build inference request
        let inference_request = InferenceRequest::from_messages(
            request.model_id.clone(),
            engine_messages,
        );

        // Ensure model is loaded
        self.models
            .ensure_model_loaded(&request.model_id, forge_engine::DeviceKind::CPU)
            .await?;

        // Submit task with High priority (interactive)
        let priority = request.priority.unwrap_or(TaskPriority::High);
        let task_id = self.scheduler.submit(
            TaskType::Inference {
                session_id: session_id.clone(),
                model_id: request.model_id.clone(),
                request: inference_request,
            },
            priority,
        )?;

        info!("Chat task submitted: {}", task_id);

        Ok(task_id)
    }

    /// Run inference directly and return the stream.
    ///
    /// This bypasses the scheduler and runs inference immediately.
    /// Useful for real-time chat where you want the stream directly.
    pub async fn run_inference(&self, model_id: &str, request: InferenceRequest) -> Result<InferenceStream> {
        debug!("Running immediate inference with model: {}", model_id);

        // Ensure model is loaded
        self.models
            .ensure_model_loaded(model_id, forge_engine::DeviceKind::CPU)
            .await?;

        // Run inference
        let stream = self.engine.run_inference(request).await?;

        Ok(stream)
    }

    /// Start a training task.
    pub async fn start_training(
        &self,
        model_id: String,
        batch: TrainBatch,
        config: TrainConfig,
    ) -> Result<String> {
        info!("Starting training task for model: {}", model_id);

        // Ensure model is loaded
        self.models
            .ensure_model_loaded(&model_id, forge_engine::DeviceKind::CPU)
            .await?;

        // Submit task with Normal priority (batch operation)
        let task_id = self.scheduler.submit(
            TaskType::Training {
                model_id: model_id.clone(),
                batch,
                config,
            },
            TaskPriority::Normal,
        )?;

        info!("Training task submitted: {}", task_id);

        Ok(task_id)
    }

    /// Start a RAG indexing task.
    pub async fn start_rag_indexing(
        &self,
        document_id: String,
        content: String,
        chunk_size: usize,
    ) -> Result<String> {
        info!("Starting RAG indexing for document: {}", document_id);

        // Submit task with Low priority (background operation)
        let task_id = self.scheduler.submit(
            TaskType::RagIndexing {
                document_id: document_id.clone(),
                content,
                chunk_size,
            },
            TaskPriority::Low,
        )?;

        info!("RAG indexing task submitted: {}", task_id);

        Ok(task_id)
    }

    /// Get job status by ID.
    pub async fn get_job_status(&self, job_id: &str) -> Result<JobStatus> {
        debug!("Getting job status: {}", job_id);

        let task = self.scheduler
            .get_task(job_id)
            .await?
            .ok_or_else(|| RuntimeError::TaskNotFound(job_id.to_string()))?;

        Ok(JobStatus::from(task))
    }

    /// List all active jobs (queued or running).
    pub fn get_active_jobs(&self) -> Vec<JobStatus> {
        let queued = self.scheduler.get_tasks_by_state(crate::tasks::TaskState::Queued);
        let running = self.scheduler.get_tasks_by_state(crate::tasks::TaskState::Running);

        queued.into_iter()
            .chain(running.into_iter())
            .map(JobStatus::from)
            .collect()
    }

    /// Register a new model.
    pub async fn register_model(&self, request: RegisterModelRequest) -> Result<String> {
        info!("Registering model: {}", request.name);

        let model_id = self.models.register_model(request).await?;

        info!("Model registered: {}", model_id);

        Ok(model_id)
    }

    // ========================================================================
    // Training Job Methods
    // ========================================================================

    /// Create a new training job.
    pub fn create_training_job(&self, config: forge_engine::TrainConfig) -> Result<String> {
        info!("Creating training job");

        let job_id = self.training_jobs
            .create_job(config)
            .map_err(|e| RuntimeError::InvalidRequest(e.to_string()))?;

        info!("Created training job: {}", job_id);

        Ok(job_id)
    }

    /// Start a training job.
    pub async fn start_training_job(&self, job_id: &str) -> Result<()> {
        info!("Starting training job: {}", job_id);

        self.training_jobs
            .start_job(job_id)
            .await
            .map_err(|e| RuntimeError::InvalidRequest(e.to_string()))?;

        Ok(())
    }

    /// Get a training job by ID.
    pub fn get_training_job(&self, job_id: &str) -> Result<forge_engine::TrainingJob> {
        self.training_jobs
            .get_job(job_id)
            .map_err(|e| RuntimeError::TaskNotFound(job_id.to_string()))
    }

    /// List all training jobs.
    pub fn list_training_jobs(&self) -> Vec<forge_engine::TrainingJob> {
        self.training_jobs.list_jobs()
    }

    /// Cancel a training job.
    pub async fn cancel_training_job(&self, job_id: &str) -> Result<()> {
        info!("Cancelling training job: {}", job_id);

        self.training_jobs
            .cancel_job(job_id)
            .await
            .map_err(|e| RuntimeError::InvalidRequest(e.to_string()))?;

        Ok(())
    }

    /// Delete a training job.
    pub fn delete_training_job(&self, job_id: &str) -> Result<()> {
        info!("Deleting training job: {}", job_id);

        self.training_jobs
            .delete_job(job_id)
            .map_err(|e| RuntimeError::InvalidRequest(e.to_string()))?;

        Ok(())
    }

    /// Get logs for a training job.
    pub fn get_training_logs(&self, job_id: &str) -> Result<Vec<String>> {
        self.training_jobs
            .get_logs(job_id)
            .map_err(|e| RuntimeError::TaskNotFound(job_id.to_string()))
    }

    /// Shutdown the runtime.
    pub fn shutdown(&self) {
        info!("Shutting down runtime");
        self.scheduler.shutdown();
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::EchoBackend;
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn setup_runtime() -> (Runtime, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        let backend = Arc::new(EchoBackend::new());
        let engine = Arc::new(Engine::new(backend));
        let models = Arc::new(ModelRegistry::with_engine(store.clone(), engine.clone()));

        let runtime = Runtime::new(store, models, engine);

        (runtime, temp_dir)
    }

    fn create_temp_model_file(temp_dir: &TempDir, name: &str) -> PathBuf {
        let path = temp_dir.path().join(name);
        std::fs::write(&path, b"fake model data").unwrap();
        path
    }

    #[tokio::test]
    async fn test_runtime_creation() {
        let (runtime, _temp_dir) = setup_runtime().await;

        // Verify components exist
        assert_eq!(runtime.models().list_models().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_register_model() {
        let (runtime, temp_dir) = setup_runtime().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let model_id = runtime.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: forge_models::Format::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        assert!(!model_id.is_empty());

        let models = runtime.models().list_models().await.unwrap();
        assert_eq!(models.len(), 1);
    }

    #[tokio::test]
    async fn test_start_chat() {
        let (runtime, temp_dir) = setup_runtime().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        // Register model
        let model_id = runtime.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: forge_models::Format::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Start chat
        let task_id = runtime.start_chat(ChatRequest {
            session_id: None,
            model_id: model_id.clone(),
            prompt: "Hello!".to_string(),
            priority: None,
        }).await.unwrap();

        assert!(!task_id.is_empty());

        // Wait for scheduler to process
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        // Get job status
        let status = runtime.get_job_status(&task_id).await.unwrap();
        assert_eq!(status.id, task_id);
    }

    #[tokio::test]
    async fn test_run_inference() {
        let (runtime, temp_dir) = setup_runtime().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        // Register model
        let model_id = runtime.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: forge_models::Format::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Run inference
        let request = InferenceRequest::from_prompt(model_id.clone(), "Test prompt".to_string());
        let stream = runtime.run_inference(&model_id, request).await.unwrap();

        // Stream is created successfully
        assert!(std::mem::size_of_val(&stream) > 0);
    }

    #[tokio::test]
    async fn test_start_training() {
        let (runtime, temp_dir) = setup_runtime().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        // Register model
        let model_id = runtime.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: forge_models::Format::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Start training
        let batch = TrainBatch {
            inputs: vec![vec![1, 2, 3]],
            targets: vec![vec![4, 5, 6]],
            masks: None,
        };

        let config = TrainConfig {
            learning_rate: 0.001,
            batch_size: 1,
            epochs: 1,
            gradient_accumulation_steps: 1,
            max_grad_norm: 1.0,
            lora_rank: None,
            lora_alpha: None,
        };

        let task_id = runtime.start_training(model_id, batch, config).await.unwrap();

        assert!(!task_id.is_empty());
    }

    #[tokio::test]
    async fn test_get_active_jobs() {
        let (runtime, temp_dir) = setup_runtime().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        // Register model
        let model_id = runtime.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: forge_models::Format::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Submit multiple tasks
        runtime.start_chat(ChatRequest {
            session_id: None,
            model_id: model_id.clone(),
            prompt: "Test 1".to_string(),
            priority: None,
        }).await.unwrap();

        runtime.start_chat(ChatRequest {
            session_id: None,
            model_id: model_id.clone(),
            prompt: "Test 2".to_string(),
            priority: None,
        }).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        let jobs = runtime.get_active_jobs();
        assert_eq!(jobs.len(), 2);
    }
}
