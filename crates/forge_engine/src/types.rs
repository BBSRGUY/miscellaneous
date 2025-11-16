//! # Core Engine Types
//!
//! Type definitions for the Forge inference and training engine.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Device kind for model execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceKind {
    /// CPU execution
    CPU,
    /// GPU execution with device ID
    GPU { id: usize },
    /// WebGPU execution with client ID
    WebGPU { client_id: String },
}

impl Default for DeviceKind {
    fn default() -> Self {
        Self::CPU
    }
}

/// Model specification for loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Unique model identifier
    pub id: String,
    /// Model name
    pub name: String,
    /// Path to model file
    pub path: PathBuf,
    /// Backend type (e.g., "llama", "echo", "candle")
    pub backend: String,
    /// Model format (e.g., "gguf", "safetensors", "pytorch")
    pub format: String,
    /// Quantization method (e.g., "Q4_K_M", "Q8_0")
    pub quantization: Option<String>,
    /// Context length in tokens
    pub context_length: usize,
    /// Device to load model on
    pub device: DeviceKind,
    /// Model parameters count
    pub parameters: Option<u64>,
}

impl ModelSpec {
    /// Create a new model specification.
    pub fn new(name: String, path: PathBuf, backend: String, format: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            path,
            backend,
            format,
            quantization: None,
            context_length: 2048,
            device: DeviceKind::default(),
            parameters: None,
        }
    }
}

/// Message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: String,
}

/// Role of a message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant/model response
    Assistant,
    /// System instruction
    System,
}

/// Inference request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Model ID to use
    pub model_id: String,
    /// Session ID for context (optional)
    pub session_id: Option<String>,
    /// Simple prompt (alternative to messages)
    pub prompt: Option<String>,
    /// Conversation messages
    pub messages: Option<Vec<Message>>,
    /// Sampling parameters
    pub sampling: SamplingParams,
}

impl InferenceRequest {
    /// Create a simple prompt-based request.
    pub fn from_prompt(model_id: String, prompt: String) -> Self {
        Self {
            model_id,
            session_id: None,
            prompt: Some(prompt),
            messages: None,
            sampling: SamplingParams::default(),
        }
    }

    /// Create a message-based request.
    pub fn from_messages(model_id: String, messages: Vec<Message>) -> Self {
        Self {
            model_id,
            session_id: None,
            prompt: None,
            messages: Some(messages),
            sampling: SamplingParams::default(),
        }
    }
}

/// Sampling parameters for text generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    /// Top-p/nucleus sampling
    pub top_p: f32,
    /// Top-k sampling
    pub top_k: Option<usize>,
    /// Repetition penalty
    pub repetition_penalty: f32,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: None,
            repetition_penalty: 1.0,
            stop_sequences: Vec::new(),
        }
    }
}

/// A single token with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token ID
    pub id: u32,
    /// Token text
    pub text: String,
    /// Log probability
    pub logprob: Option<f32>,
}

/// Streaming inference chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceChunk {
    /// Generated token
    pub token: Option<Token>,
    /// Generated text (accumulated or delta)
    pub text: String,
    /// Finish reason (if complete)
    pub finish_reason: Option<FinishReason>,
    /// Generation index
    pub index: usize,
}

/// Reason why generation finished.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Reached max tokens
    MaxTokens,
    /// Hit stop sequence
    StopSequence,
    /// Model indicated end of text
    EndOfText,
    /// Stopped by user/system
    Cancelled,
}

/// Training batch for fine-tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainBatch {
    /// Input sequences
    pub inputs: Vec<Vec<u32>>,
    /// Target sequences
    pub targets: Vec<Vec<u32>>,
    /// Attention masks
    pub masks: Option<Vec<Vec<bool>>>,
}

/// Training configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    /// Learning rate
    pub learning_rate: f32,
    /// Batch size
    pub batch_size: usize,
    /// Number of training epochs
    pub epochs: usize,
    /// Gradient accumulation steps
    pub gradient_accumulation_steps: usize,
    /// Maximum gradient norm for clipping
    pub max_grad_norm: f32,
    /// LoRA rank (if using LoRA)
    pub lora_rank: Option<usize>,
    /// LoRA alpha
    pub lora_alpha: Option<f32>,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-4,
            batch_size: 1,
            epochs: 1,
            gradient_accumulation_steps: 1,
            max_grad_norm: 1.0,
            lora_rank: Some(8),
            lora_alpha: Some(16.0),
        }
    }
}

/// Result from a single training step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainStepResult {
    /// Step number
    pub step: usize,
    /// Training loss
    pub loss: f32,
    /// Learning rate used
    pub learning_rate: f32,
    /// Gradient norm
    pub grad_norm: Option<f32>,
    /// Tokens processed in this step
    pub tokens_processed: usize,
}

/// Handle to a loaded model.
#[derive(Debug, Clone)]
pub struct ModelHandle {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Backend type
    pub backend: String,
}

impl ModelHandle {
    /// Create a new model handle.
    pub fn new(id: String, name: String, backend: String) -> Self {
        Self { id, name, backend }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_spec_creation() {
        let spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.gguf"),
            "llama".to_string(),
            "gguf".to_string(),
        );

        assert_eq!(spec.name, "test-model");
        assert_eq!(spec.context_length, 2048);
        assert_eq!(spec.device, DeviceKind::CPU);
    }

    #[test]
    fn test_inference_request_from_prompt() {
        let request = InferenceRequest::from_prompt(
            "model-1".to_string(),
            "Hello".to_string(),
        );

        assert!(request.prompt.is_some());
        assert!(request.messages.is_none());
        assert_eq!(request.sampling.max_tokens, 512);
    }

    #[test]
    fn test_inference_request_from_messages() {
        let messages = vec![
            Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
        ];

        let request = InferenceRequest::from_messages(
            "model-1".to_string(),
            messages.clone(),
        );

        assert!(request.prompt.is_none());
        assert_eq!(request.messages.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_device_kind() {
        let cpu = DeviceKind::CPU;
        let gpu = DeviceKind::GPU { id: 0 };
        let webgpu = DeviceKind::WebGPU {
            client_id: "client-1".to_string(),
        };

        assert_eq!(cpu, DeviceKind::default());
        assert_ne!(cpu, gpu);
        assert_ne!(gpu, webgpu);
    }

    #[test]
    fn test_sampling_params_default() {
        let params = SamplingParams::default();
        assert_eq!(params.max_tokens, 512);
        assert_eq!(params.temperature, 0.7);
        assert_eq!(params.top_p, 0.9);
    }
}
