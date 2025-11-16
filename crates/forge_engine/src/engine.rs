//! # Inference Engine
//!
//! High-level engine for managing models and running inference.

use crate::backend::{InferenceStream, ModelBackend, ModelInfo};
use crate::types::*;
use crate::{EngineError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Main inference engine.
///
/// The engine manages a collection of loaded models and provides
/// a unified interface for inference and training operations.
pub struct Engine {
    backend: Arc<dyn ModelBackend>,
    models: Arc<RwLock<HashMap<String, ModelHandle>>>,
}

impl Engine {
    /// Create a new engine with the given backend.
    pub fn new(backend: Arc<dyn ModelBackend>) -> Self {
        info!("Initializing inference engine");
        Self {
            backend,
            models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a model from specification.
    ///
    /// This loads the model using the backend and registers it
    /// in the engine's model registry.
    pub async fn load_model(&self, spec: &ModelSpec) -> Result<ModelHandle> {
        info!("Loading model: {} (backend: {})", spec.name, spec.backend);

        // Load the model through the backend
        let handle = self.backend.load_model(spec).await?;

        // Register in our local registry
        let mut models = self.models.write();
        models.insert(handle.id.clone(), handle.clone());

        info!("Model loaded successfully: {}", handle.name);
        Ok(handle)
    }

    /// Unload a model and free its resources.
    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        info!("Unloading model: {}", model_id);

        // Unload from backend
        self.backend.unload_model(model_id).await?;

        // Remove from registry
        let mut models = self.models.write();
        if models.remove(model_id).is_some() {
            info!("Model unloaded successfully: {}", model_id);
            Ok(())
        } else {
            warn!("Model not found in registry: {}", model_id);
            Err(EngineError::ModelNotLoaded(model_id.to_string()))
        }
    }

    /// List all loaded models.
    pub fn list_models(&self) -> Vec<ModelHandle> {
        let models = self.models.read();
        models.values().cloned().collect()
    }

    /// Check if a model is loaded.
    pub fn is_model_loaded(&self, model_id: &str) -> bool {
        self.backend.is_loaded(model_id)
    }

    /// Get information about a loaded model.
    pub fn model_info(&self, model_id: &str) -> Option<ModelInfo> {
        self.backend.model_info(model_id)
    }

    /// Run inference on a model.
    ///
    /// Returns a stream of inference chunks that can be consumed
    /// as the model generates tokens.
    pub async fn run_inference(&self, request: InferenceRequest) -> Result<InferenceStream> {
        debug!(
            "Running inference on model: {}",
            request.model_id
        );

        // Verify model is loaded
        if !self.is_model_loaded(&request.model_id) {
            return Err(EngineError::ModelNotLoaded(request.model_id.clone()));
        }

        // Run inference through backend
        self.backend.infer(request).await
    }

    /// Run a training step.
    pub async fn train_step(
        &self,
        model_id: &str,
        batch: TrainBatch,
        config: &TrainConfig,
    ) -> Result<TrainStepResult> {
        debug!("Running training step on model: {}", model_id);

        // Verify model is loaded
        if !self.is_model_loaded(model_id) {
            return Err(EngineError::ModelNotLoaded(model_id.to_string()));
        }

        // Run training step through backend
        self.backend.train_step(model_id, batch, config).await
    }

    /// Get the number of loaded models.
    pub fn model_count(&self) -> usize {
        let models = self.models.read();
        models.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::echo::EchoBackend;
    use futures::StreamExt;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_engine_load_model() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        let handle = engine.load_model(&spec).await.unwrap();
        assert_eq!(handle.name, "test-model");
        assert_eq!(engine.model_count(), 1);
    }

    #[tokio::test]
    async fn test_engine_list_models() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let spec1 = ModelSpec::new(
            "model-1".to_string(),
            PathBuf::from("model1.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        let spec2 = ModelSpec::new(
            "model-2".to_string(),
            PathBuf::from("model2.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );

        engine.load_model(&spec1).await.unwrap();
        engine.load_model(&spec2).await.unwrap();

        let models = engine.list_models();
        assert_eq!(models.len(), 2);
    }

    #[tokio::test]
    async fn test_engine_unload_model() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let mut spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();

        engine.load_model(&spec).await.unwrap();
        assert!(engine.is_model_loaded("model-1"));

        engine.unload_model("model-1").await.unwrap();
        assert!(!engine.is_model_loaded("model-1"));
        assert_eq!(engine.model_count(), 0);
    }

    #[tokio::test]
    async fn test_engine_inference() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let mut spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();

        engine.load_model(&spec).await.unwrap();

        let request = InferenceRequest::from_prompt(
            "model-1".to_string(),
            "Hello, world!".to_string(),
        );

        let mut stream = engine.run_inference(request).await.unwrap();

        let mut chunk_count = 0;
        while let Some(chunk_result) = stream.next().await {
            chunk_result.unwrap();
            chunk_count += 1;
        }

        assert!(chunk_count > 0);
    }

    #[tokio::test]
    async fn test_engine_model_not_loaded() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let request = InferenceRequest::from_prompt(
            "non-existent".to_string(),
            "Hello".to_string(),
        );

        let result = engine.run_inference(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_engine_model_info() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let mut spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();
        spec.parameters = Some(7_000_000_000);

        engine.load_model(&spec).await.unwrap();

        let info = engine.model_info("model-1").unwrap();
        assert_eq!(info.name, "test-model");
        assert_eq!(info.backend, "echo");
        assert_eq!(info.parameters, 7_000_000_000);
    }

    #[tokio::test]
    async fn test_engine_training() {
        let backend = Arc::new(EchoBackend::new());
        let engine = Engine::new(backend);

        let mut spec = ModelSpec::new(
            "test-model".to_string(),
            PathBuf::from("model.bin"),
            "echo".to_string(),
            "bin".to_string(),
        );
        spec.id = "model-1".to_string();

        engine.load_model(&spec).await.unwrap();

        let batch = TrainBatch {
            inputs: vec![vec![1, 2, 3]],
            targets: vec![vec![2, 3, 4]],
            masks: None,
        };

        let config = TrainConfig::default();

        let result = engine
            .train_step("model-1", batch, &config)
            .await
            .unwrap();

        assert!(result.loss > 0.0);
        assert_eq!(result.tokens_processed, 3);
    }
}
