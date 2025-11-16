//! # Forge Engine
//!
//! Model backends, inference, and training interfaces for the Forge platform.
//!
//! This crate provides the core abstractions for running LLMs locally,
//! including inference with streaming, training (LoRA/QLoRA), and model execution.
//!
//! ## Architecture
//!
//! The engine is organized into several modules:
//!
//! - `types` - Core type definitions (DeviceKind, ModelSpec, InferenceRequest, etc.)
//! - `backend` - ModelBackend trait for implementing model backends
//! - `echo` - Dummy echo backend for testing
//! - `engine` - High-level Engine for managing models and running operations
//!
//! ## Example
//!
//! ```no_run
//! use forge_engine::{Engine, EchoBackend, ModelSpec, InferenceRequest};
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create engine with echo backend
//!     let backend = Arc::new(EchoBackend::new());
//!     let engine = Engine::new(backend);
//!
//!     // Load a model
//!     let spec = ModelSpec::new(
//!         "my-model".to_string(),
//!         PathBuf::from("model.bin"),
//!         "echo".to_string(),
//!         "bin".to_string(),
//!     );
//!     let handle = engine.load_model(&spec).await?;
//!
//!     // Run inference with streaming
//!     let request = InferenceRequest::from_prompt(
//!         handle.id.clone(),
//!         "Hello, world!".to_string(),
//!     );
//!
//!     let mut stream = engine.run_inference(request).await?;
//!     while let Some(chunk) = stream.next().await {
//!         let chunk = chunk?;
//!         print!("{}", chunk.text);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod backend;
pub mod echo;
pub mod engine;
pub mod types;

use thiserror::Error;

// Re-export commonly used types
pub use backend::{InferenceStream, ModelBackend, ModelInfo};
pub use echo::EchoBackend;
pub use engine::Engine;
pub use types::*;

/// Errors that can occur during model inference or training.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("Training failed: {0}")]
    TrainingFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_end_to_end_workflow() {
        // Create engine with echo backend
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        // Load a model
        let spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        let handle = engine.load_model(&spec).await.unwrap();

        // Run inference
        let request = InferenceRequest::from_prompt(
            handle.id.clone(),
            "Test prompt".to_string(),
        );

        let mut stream = engine.run_inference(request).await.unwrap();

        let mut text = String::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            text.push_str(&chunk.text);
        }

        assert!(text.contains("Echo:"));
        assert!(text.contains("Test prompt"));

        // Unload model
        engine.unload_model(&handle.id).await.unwrap();
        assert_eq!(engine.model_count(), 0);
    }

    #[tokio::test]
    async fn test_message_based_inference() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        let handle = engine.load_model(&spec).await.unwrap();

        // Create conversation
        let messages = vec![
            Message {
                role: MessageRole::System,
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: "Hello!".to_string(),
            },
        ];

        let request = InferenceRequest::from_messages(handle.id.clone(), messages);

        let mut stream = engine.run_inference(request).await.unwrap();

        let mut got_response = false;
        while let Some(chunk_result) = stream.next().await {
            chunk_result.unwrap();
            got_response = true;
        }

        assert!(got_response);
    }

    #[tokio::test]
    async fn test_training_workflow() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        let handle = engine.load_model(&spec).await.unwrap();

        // Create training batch
        let batch = TrainBatch {
            inputs: vec![vec![1, 2, 3, 4, 5]],
            targets: vec![vec![2, 3, 4, 5, 6]],
            masks: None,
        };

        let config = TrainConfig {
            learning_rate: 1e-4,
            batch_size: 1,
            epochs: 1,
            gradient_accumulation_steps: 1,
            max_grad_norm: 1.0,
            lora_rank: Some(8),
            lora_alpha: Some(16.0),
        };

        let result = engine
            .train_step(&handle.id, batch, &config)
            .await
            .unwrap();

        assert!(result.loss > 0.0);
        assert_eq!(result.learning_rate, 1e-4);
        assert_eq!(result.tokens_processed, 5);
    }
}
