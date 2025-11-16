//! # Model Backend Trait
//!
//! Core trait for model backends with inference and training support.

use crate::types::*;
use crate::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

/// Type alias for inference stream.
pub type InferenceStream = Pin<Box<dyn Stream<Item = Result<InferenceChunk>> + Send>>;

/// Model backend trait for inference and training.
#[async_trait]
pub trait ModelBackend: Send + Sync {
    /// Load a model from specification.
    ///
    /// This initializes the model and returns a handle that can be used
    /// for inference and training operations.
    async fn load_model(&self, spec: &ModelSpec) -> Result<ModelHandle>;

    /// Run inference with streaming output.
    ///
    /// Returns a stream of inference chunks that can be consumed
    /// as tokens are generated.
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceStream>;

    /// Run a single training step.
    ///
    /// Processes a batch of training data and returns metrics.
    async fn train_step(
        &self,
        model_id: &str,
        batch: TrainBatch,
        config: &TrainConfig,
    ) -> Result<TrainStepResult>;

    /// Unload a model and free resources.
    async fn unload_model(&self, model_id: &str) -> Result<()>;

    /// Check if a model is currently loaded.
    fn is_loaded(&self, model_id: &str) -> bool;

    /// Get information about a loaded model.
    fn model_info(&self, model_id: &str) -> Option<ModelInfo>;
}

/// Information about a loaded model.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// Backend type
    pub backend: String,
    /// Number of parameters
    pub parameters: u64,
    /// Context length
    pub context_length: usize,
    /// Device the model is loaded on
    pub device: DeviceKind,
}

impl ModelInfo {
    /// Create new model info.
    pub fn new(
        id: String,
        name: String,
        backend: String,
        parameters: u64,
        context_length: usize,
    ) -> Self {
        Self {
            id,
            name,
            backend,
            parameters,
            context_length,
            device: DeviceKind::CPU,
        }
    }
}
