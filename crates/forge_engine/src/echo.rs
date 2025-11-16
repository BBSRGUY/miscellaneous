//! # Echo Backend
//!
//! Dummy backend implementation for testing and development.

use crate::backend::{InferenceStream, ModelBackend, ModelInfo};
use crate::types::*;
use crate::{EngineError, Result};
use async_trait::async_trait;
use futures::stream;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Echo backend that returns deterministic responses.
///
/// This is a dummy backend used for testing. It:
/// - Simulates model loading
/// - Returns echo responses for inference
/// - Supports streaming with artificial delays
/// - Simulates training steps
pub struct EchoBackend {
    models: Arc<RwLock<HashMap<String, LoadedModel>>>,
}

struct LoadedModel {
    spec: ModelSpec,
}

impl EchoBackend {
    /// Create a new echo backend.
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for EchoBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelBackend for EchoBackend {
    async fn load_model(&self, spec: &ModelSpec) -> Result<ModelHandle> {
        // Simulate loading delay
        sleep(Duration::from_millis(100)).await;

        let handle = ModelHandle::new(
            spec.id.clone(),
            spec.name.clone(),
            "echo".to_string(),
        );

        let mut models = self.models.write();
        models.insert(
            spec.id.clone(),
            LoadedModel {
                spec: spec.clone(),
            },
        );

        Ok(handle)
    }

    async fn infer(&self, request: InferenceRequest) -> Result<InferenceStream> {
        // Check if model is loaded
        let model_id = request.model_id.clone();
        {
            let models = self.models.read();
            if !models.contains_key(&model_id) {
                return Err(EngineError::ModelNotLoaded(model_id));
            }
        }

        // Get the prompt
        let prompt = if let Some(p) = request.prompt {
            p
        } else if let Some(messages) = request.messages {
            // Concatenate messages
            messages
                .iter()
                .map(|m| format!("{}: {}", format!("{:?}", m.role), m.content))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            return Err(EngineError::InvalidConfig(
                "Either prompt or messages must be provided".to_string(),
            ));
        };

        // Create echo response
        let response_text = format!("Echo: {}", prompt);
        let max_tokens = request.sampling.max_tokens;

        // Create streaming response
        let chunks: Vec<Result<InferenceChunk>> = response_text
            .chars()
            .take(max_tokens)
            .enumerate()
            .map(|(i, c)| {
                let is_last = i == response_text.chars().count() - 1 || i == max_tokens - 1;
                Ok(InferenceChunk {
                    token: Some(Token {
                        id: c as u32,
                        text: c.to_string(),
                        logprob: Some(-0.1),
                    }),
                    text: c.to_string(),
                    finish_reason: if is_last {
                        Some(if i == max_tokens - 1 {
                            FinishReason::MaxTokens
                        } else {
                            FinishReason::EndOfText
                        })
                    } else {
                        None
                    },
                    index: i,
                })
            })
            .collect();

        // Create async stream with delays to simulate streaming
        let stream = stream::iter(chunks);

        // Add small delays between chunks to simulate streaming
        let delayed_stream = stream::unfold((stream, 0), |(mut stream, count)| async move {
            use futures::StreamExt;

            if count > 0 {
                // Add delay between chunks (except first one)
                sleep(Duration::from_millis(10)).await;
            }

            stream.next().await.map(|chunk| (chunk, (stream, count + 1)))
        });

        Ok(Box::pin(delayed_stream))
    }

    async fn train_step(
        &self,
        model_id: &str,
        batch: TrainBatch,
        config: &TrainConfig,
    ) -> Result<TrainStepResult> {
        // Check if model is loaded
        {
            let models = self.models.read();
            if !models.contains_key(model_id) {
                return Err(EngineError::ModelNotLoaded(model_id.to_string()));
            }
        }

        // Simulate training delay
        sleep(Duration::from_millis(50)).await;

        // Return dummy training metrics
        let tokens_in_batch: usize = batch.inputs.iter().map(|seq| seq.len()).sum();

        Ok(TrainStepResult {
            step: 0, // Caller should track this
            loss: 2.5 - (0.1 * (tokens_in_batch as f32 / 100.0)), // Fake decreasing loss
            learning_rate: config.learning_rate,
            grad_norm: Some(0.5),
            tokens_processed: tokens_in_batch,
        })
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        let mut models = self.models.write();

        if models.remove(model_id).is_some() {
            Ok(())
        } else {
            Err(EngineError::ModelNotLoaded(model_id.to_string()))
        }
    }

    fn is_loaded(&self, model_id: &str) -> bool {
        let models = self.models.read();
        models.contains_key(model_id)
    }

    fn model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let models = self.models.read();
        models.get(model_id).map(|loaded| ModelInfo {
            id: loaded.spec.id.clone(),
            name: loaded.spec.name.clone(),
            backend: "echo".to_string(),
            parameters: loaded.spec.parameters.unwrap_or(0),
            context_length: loaded.spec.context_length,
            device: loaded.spec.device.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_echo_backend_load_model() {
        let backend = EchoBackend::new();
        let spec = ModelSpec::new(
            "echo-model".to_string(),
            PathBuf::from("dummy.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        let handle = backend.load_model(&spec).await.unwrap();
        assert_eq!(handle.name, "echo-model");
        assert!(backend.is_loaded(&spec.id));
    }

    #[tokio::test]
    async fn test_echo_backend_inference_streaming() {
        let backend = EchoBackend::new();
        let mut spec = ModelSpec::new(
            "echo-model".to_string(),
            PathBuf::from("dummy.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();

        backend.load_model(&spec).await.unwrap();

        let request = InferenceRequest::from_prompt(
            "model-1".to_string(),
            "Hello".to_string(),
        );

        let mut stream = backend.infer(request).await.unwrap();

        let mut chunks = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            chunks.push(chunk);
        }

        assert!(!chunks.is_empty());

        // Last chunk should have finish_reason
        let last_chunk = chunks.last().unwrap();
        assert!(last_chunk.finish_reason.is_some());
    }

    #[tokio::test]
    async fn test_echo_backend_training() {
        let backend = EchoBackend::new();
        let mut spec = ModelSpec::new(
            "echo-model".to_string(),
            PathBuf::from("dummy.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();

        backend.load_model(&spec).await.unwrap();

        let batch = TrainBatch {
            inputs: vec![vec![1, 2, 3], vec![4, 5, 6]],
            targets: vec![vec![2, 3, 4], vec![5, 6, 7]],
            masks: None,
        };

        let config = TrainConfig::default();

        let result = backend
            .train_step("model-1", batch, &config)
            .await
            .unwrap();

        assert!(result.loss > 0.0);
        assert_eq!(result.tokens_processed, 6);
    }

    #[tokio::test]
    async fn test_echo_backend_unload() {
        let backend = EchoBackend::new();
        let mut spec = ModelSpec::new(
            "echo-model".to_string(),
            PathBuf::from("dummy.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();

        backend.load_model(&spec).await.unwrap();
        assert!(backend.is_loaded("model-1"));

        backend.unload_model("model-1").await.unwrap();
        assert!(!backend.is_loaded("model-1"));
    }

    #[tokio::test]
    async fn test_echo_backend_model_not_loaded() {
        let backend = EchoBackend::new();

        let request = InferenceRequest::from_prompt(
            "non-existent".to_string(),
            "Hello".to_string(),
        );

        let result = backend.infer(request).await;
        assert!(matches!(result, Err(EngineError::ModelNotLoaded(_))));
    }
}
