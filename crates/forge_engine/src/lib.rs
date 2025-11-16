//! # Forge Engine
//!
//! Model backends, inference, and training interfaces for the Forge platform.
//!
//! This crate provides the core abstractions for running LLMs locally,
//! including inference, training (LoRA/QLoRA), and model execution.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EngineError>;

/// Request for text generation/completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

/// Response from text generation/completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub tokens_generated: usize,
    pub finish_reason: FinishReason,
}

/// Reason why text generation stopped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    /// Reached maximum token limit
    MaxTokens,
    /// Hit a stop sequence
    StopSequence,
    /// Model indicated end of generation
    EndOfText,
}

/// Trait for inference backends.
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Generate text completion from a prompt.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Get model information.
    fn model_info(&self) -> ModelInfo;
}

/// Information about a loaded model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub backend: String,
    pub parameters: u64,
    pub context_length: usize,
}

/// Simple echo backend for testing (returns the prompt as completion).
pub struct EchoBackend {
    model_name: String,
}

impl EchoBackend {
    pub fn new(model_name: String) -> Self {
        Self { model_name }
    }
}

#[async_trait]
impl InferenceBackend for EchoBackend {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let max_tokens = request.max_tokens.unwrap_or(100);
        let response_text = format!("Echo: {}", request.prompt);

        Ok(CompletionResponse {
            text: response_text.chars().take(max_tokens).collect(),
            tokens_generated: max_tokens.min(response_text.len()),
            finish_reason: FinishReason::MaxTokens,
        })
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.model_name.clone(),
            backend: "echo".to_string(),
            parameters: 0,
            context_length: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_backend() {
        let backend = EchoBackend::new("test-model".to_string());
        let request = CompletionRequest {
            prompt: "Hello, world!".to_string(),
            max_tokens: Some(50),
            temperature: None,
            top_p: None,
            stop_sequences: None,
        };

        let response = backend.complete(request).await.unwrap();
        assert!(response.text.starts_with("Echo:"));
        assert!(response.text.contains("Hello, world!"));
    }
}
