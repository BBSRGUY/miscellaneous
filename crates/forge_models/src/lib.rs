//! # Forge Models
//!
//! Model registry, loading, and metadata management with database integration.
//!
//! This crate manages the catalog of available models, their installation,
//! loading state, and metadata. It bridges the storage layer (forge_store)
//! with the engine layer (forge_engine).

pub mod types;

use chrono::DateTime;
use forge_engine::{DeviceKind, Engine, ModelSpec};
use forge_store::{Model, Store, StoreError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};
use types::{ModelFormat, QuantizationType};

pub use types::{ModelFormat as Format, QuantizationType as Quantization};

/// Errors that can occur during model operations.
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Model already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid model metadata: {0}")]
    InvalidMetadata(String),

    #[error("Model path does not exist: {0}")]
    PathNotFound(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StoreError),

    #[error("Engine error: {0}")]
    Engine(#[from] forge_engine::EngineError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ModelError>;

/// High-level model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Model name
    pub name: String,
    /// File path to model weights
    pub path: Option<PathBuf>,
    /// Backend type (e.g., "echo", "llama", "candle")
    pub backend: String,
    /// Model format
    pub format: ModelFormat,
    /// Quantization type
    pub quantization: QuantizationType,
    /// Model size in bytes
    pub size_bytes: Option<u64>,
    /// Model tags
    pub tags: Vec<String>,
    /// Whether the model is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<chrono::Utc>,
}

impl ModelInfo {
    /// Create from database Model entity.
    pub fn from_db_model(model: Model) -> Result<Self> {
        let format = ModelFormat::from_str(&model.format)
            .unwrap_or(ModelFormat::Other);

        let quantization = model.quantization
            .as_ref()
            .and_then(|q| QuantizationType::from_str(q).ok())
            .unwrap_or(QuantizationType::None);

        let tags = model.parse_tags();
        let enabled = !tags.iter().any(|t| t == "disabled");

        let created_at = DateTime::parse_from_rfc3339(&model.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| ModelError::InvalidMetadata(format!("Invalid created_at: {}", e)))?;

        let updated_at = DateTime::parse_from_rfc3339(&model.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| ModelError::InvalidMetadata(format!("Invalid updated_at: {}", e)))?;

        Ok(Self {
            id: model.id,
            name: model.name,
            path: model.path.map(PathBuf::from),
            backend: model.backend,
            format,
            quantization,
            size_bytes: model.size_bytes.map(|s| s as u64),
            tags,
            enabled,
            created_at,
            updated_at,
        })
    }

    /// Convert to database Model entity.
    pub fn to_db_model(&self) -> Model {
        let mut model = Model::new(
            self.name.clone(),
            self.backend.clone(),
            self.format.to_string(),
        );
        model.id = self.id.clone();
        model.path = self.path.as_ref().map(|p| p.display().to_string());
        model.quantization = Some(self.quantization.to_string());
        model.size_bytes = self.size_bytes.map(|s| s as i64);
        model.created_at = self.created_at.to_rfc3339();
        model.updated_at = self.updated_at.to_rfc3339();

        // Add disabled tag if not enabled
        let mut tags = self.tags.clone();
        if !self.enabled && !tags.contains(&"disabled".to_string()) {
            tags.push("disabled".to_string());
        } else if self.enabled {
            tags.retain(|t| t != "disabled");
        }
        model.set_tags(tags);

        model
    }
}

/// Request to register a new model.
#[derive(Debug, Clone)]
pub struct RegisterModelRequest {
    /// Model name (must be unique)
    pub name: String,
    /// Path to model file
    pub path: PathBuf,
    /// Backend type
    pub backend: String,
    /// Model format
    pub format: ModelFormat,
    /// Quantization type
    pub quantization: Option<QuantizationType>,
    /// Optional tags
    pub tags: Vec<String>,
}

/// Model registry that manages model metadata and lifecycle.
///
/// The registry bridges the storage layer (forge_store) with the engine layer
/// (forge_engine). It manages model metadata in the database and coordinates
/// with the engine for loading models.
pub struct ModelRegistry {
    store: Arc<Store>,
    engine: Option<Arc<Engine>>,
}

impl ModelRegistry {
    /// Create a new model registry.
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
            engine: None,
        }
    }

    /// Create a new model registry with an engine for loading models.
    pub fn with_engine(store: Arc<Store>, engine: Arc<Engine>) -> Self {
        Self {
            store,
            engine: Some(engine),
        }
    }

    /// List all models from the database.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        debug!("Listing all models from database");
        let models = self.store.models().list().await?;

        let model_infos: Vec<ModelInfo> = models
            .into_iter()
            .filter_map(|m| match ModelInfo::from_db_model(m) {
                Ok(info) => Some(info),
                Err(e) => {
                    warn!("Failed to convert model: {}", e);
                    None
                }
            })
            .collect();

        debug!("Found {} models", model_infos.len());
        Ok(model_infos)
    }

    /// Get a model by ID.
    pub async fn get_model(&self, id: &str) -> Result<ModelInfo> {
        debug!("Getting model by ID: {}", id);
        let model = self.store.models().get(id).await
            .map_err(|_| ModelError::NotFound(id.to_string()))?;
        ModelInfo::from_db_model(model)
    }

    /// Get a model by name.
    pub async fn get_model_by_name(&self, name: &str) -> Result<ModelInfo> {
        debug!("Getting model by name: {}", name);
        let model = self.store.models().get_by_name(name).await
            .map_err(|_| ModelError::NotFound(name.to_string()))?;
        ModelInfo::from_db_model(model)
    }

    /// Register a new model in the database.
    pub async fn register_model(&self, request: RegisterModelRequest) -> Result<String> {
        info!("Registering new model: {}", request.name);

        // Validate that the path exists
        if !request.path.exists() {
            return Err(ModelError::PathNotFound(request.path.display().to_string()));
        }

        // Check if model with this name already exists
        if let Ok(_) = self.store.models().get_by_name(&request.name).await {
            return Err(ModelError::AlreadyExists(request.name));
        }

        // Create model entity
        let mut model = Model::new(
            request.name.clone(),
            request.backend,
            request.format.to_string(),
        );
        model.path = Some(request.path.display().to_string());
        model.quantization = request.quantization.map(|q| q.to_string());

        // Get file size
        if let Ok(metadata) = std::fs::metadata(&request.path) {
            model.size_bytes = Some(metadata.len() as i64);
        }

        // Set tags
        model.set_tags(request.tags);

        // Save to database
        self.store.models().create(&model).await?;

        info!("Model registered successfully: {} (ID: {})", model.name, model.id);
        Ok(model.id)
    }

    /// Enable a model (remove "disabled" tag).
    pub async fn enable_model(&self, id: &str) -> Result<()> {
        debug!("Enabling model: {}", id);
        let mut model = self.store.models().get(id).await
            .map_err(|_| ModelError::NotFound(id.to_string()))?;

        let mut tags = model.parse_tags();
        tags.retain(|t| t != "disabled");
        model.set_tags(tags);
        model.updated_at = chrono::Utc::now().to_rfc3339();

        self.store.models().update(&model).await?;
        info!("Model enabled: {}", id);
        Ok(())
    }

    /// Disable a model (add "disabled" tag).
    pub async fn disable_model(&self, id: &str) -> Result<()> {
        debug!("Disabling model: {}", id);
        let mut model = self.store.models().get(id).await
            .map_err(|_| ModelError::NotFound(id.to_string()))?;

        let mut tags = model.parse_tags();
        if !tags.contains(&"disabled".to_string()) {
            tags.push("disabled".to_string());
        }
        model.set_tags(tags);
        model.updated_at = chrono::Utc::now().to_rfc3339();

        self.store.models().update(&model).await?;
        info!("Model disabled: {}", id);
        Ok(())
    }

    /// Delete a model from the database.
    pub async fn delete_model(&self, id: &str) -> Result<()> {
        info!("Deleting model: {}", id);
        self.store.models().delete(id).await
            .map_err(|_| ModelError::NotFound(id.to_string()))?;
        info!("Model deleted: {}", id);
        Ok(())
    }

    /// Build a ModelSpec from a database model entry.
    ///
    /// This converts the stored model metadata into an engine-compatible
    /// ModelSpec that can be used for loading.
    pub async fn build_model_spec(&self, id: &str, device: DeviceKind) -> Result<ModelSpec> {
        debug!("Building ModelSpec for model: {}", id);

        let model_info = self.get_model(id).await?;

        // Validate that path exists
        let path = model_info.path
            .ok_or_else(|| ModelError::InvalidMetadata("Model has no path".to_string()))?;

        if !path.exists() {
            return Err(ModelError::PathNotFound(path.display().to_string()));
        }

        // Build ModelSpec
        let mut spec = ModelSpec::new(
            model_info.name,
            path,
            model_info.backend,
            model_info.format.to_string(),
        );
        spec.id = model_info.id;
        spec.quantization = Some(model_info.quantization.to_string());
        spec.device = device;
        spec.parameters = model_info.size_bytes;

        Ok(spec)
    }

    /// Ensure a model is loaded in the engine.
    ///
    /// If the model is not already loaded, this will load it using the
    /// configured engine. Returns an error if no engine is configured.
    pub async fn ensure_model_loaded(&self, id: &str, device: DeviceKind) -> Result<()> {
        let engine = self.engine.as_ref()
            .ok_or_else(|| ModelError::InvalidMetadata("No engine configured".to_string()))?;

        // Check if already loaded
        if engine.is_model_loaded(id) {
            debug!("Model already loaded: {}", id);
            return Ok(());
        }

        info!("Loading model into engine: {}", id);

        // Build spec and load
        let spec = self.build_model_spec(id, device).await?;
        engine.load_model(&spec).await?;

        info!("Model loaded successfully: {}", id);
        Ok(())
    }

    /// Unload a model from the engine.
    pub async fn unload_model(&self, id: &str) -> Result<()> {
        let engine = self.engine.as_ref()
            .ok_or_else(|| ModelError::InvalidMetadata("No engine configured".to_string()))?;

        info!("Unloading model from engine: {}", id);
        engine.unload_model(id).await?;
        info!("Model unloaded successfully: {}", id);
        Ok(())
    }

    /// Check if a model is currently loaded in the engine.
    pub fn is_model_loaded(&self, id: &str) -> bool {
        self.engine
            .as_ref()
            .map(|e| e.is_model_loaded(id))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::EchoBackend;
    use tempfile::TempDir;

    async fn setup_test_registry() -> (ModelRegistry, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        let registry = ModelRegistry::new(store);
        (registry, temp_dir)
    }

    async fn setup_test_registry_with_engine() -> (ModelRegistry, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        let backend = Arc::new(EchoBackend::new());
        let engine = Arc::new(Engine::new(backend));
        let registry = ModelRegistry::with_engine(store, engine);
        (registry, temp_dir)
    }

    fn create_temp_model_file(temp_dir: &TempDir, name: &str) -> PathBuf {
        let path = temp_dir.path().join(name);
        std::fs::write(&path, b"fake model data").unwrap();
        path
    }

    #[tokio::test]
    async fn test_register_model() {
        let (registry, temp_dir) = setup_test_registry().await;
        let model_path = create_temp_model_file(&temp_dir, "test-model.gguf");

        let request = RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: Some(QuantizationType::Q4_K_M),
            tags: vec!["test".to_string()],
        };

        let id = registry.register_model(request).await.unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_list_models() {
        let (registry, temp_dir) = setup_test_registry().await;

        let model1_path = create_temp_model_file(&temp_dir, "model1.gguf");
        let model2_path = create_temp_model_file(&temp_dir, "model2.gguf");

        registry.register_model(RegisterModelRequest {
            name: "model-1".to_string(),
            path: model1_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        registry.register_model(RegisterModelRequest {
            name: "model-2".to_string(),
            path: model2_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        let models = registry.list_models().await.unwrap();
        assert_eq!(models.len(), 2);
    }

    #[tokio::test]
    async fn test_get_model() {
        let (registry, temp_dir) = setup_test_registry().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let id = registry.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: Some(QuantizationType::Q4_K_M),
            tags: vec![],
        }).await.unwrap();

        let model = registry.get_model(&id).await.unwrap();
        assert_eq!(model.name, "test-model");
        assert_eq!(model.format, ModelFormat::GGUF);
        assert_eq!(model.quantization, QuantizationType::Q4_K_M);
    }

    #[tokio::test]
    async fn test_enable_disable_model() {
        let (registry, temp_dir) = setup_test_registry().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let id = registry.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Initially enabled
        let model = registry.get_model(&id).await.unwrap();
        assert!(model.enabled);

        // Disable
        registry.disable_model(&id).await.unwrap();
        let model = registry.get_model(&id).await.unwrap();
        assert!(!model.enabled);

        // Re-enable
        registry.enable_model(&id).await.unwrap();
        let model = registry.get_model(&id).await.unwrap();
        assert!(model.enabled);
    }

    #[tokio::test]
    async fn test_build_model_spec() {
        let (registry, temp_dir) = setup_test_registry().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let id = registry.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path.clone(),
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: Some(QuantizationType::Q4_K_M),
            tags: vec![],
        }).await.unwrap();

        let spec = registry.build_model_spec(&id, DeviceKind::CPU).await.unwrap();
        assert_eq!(spec.name, "test-model");
        assert_eq!(spec.backend, "echo");
        assert_eq!(spec.path, model_path);
        assert_eq!(spec.device, DeviceKind::CPU);
    }

    #[tokio::test]
    async fn test_ensure_model_loaded() {
        let (registry, temp_dir) = setup_test_registry_with_engine().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let id = registry.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: Some(QuantizationType::Q4_K_M),
            tags: vec![],
        }).await.unwrap();

        // Initially not loaded
        assert!(!registry.is_model_loaded(&id));

        // Load it
        registry.ensure_model_loaded(&id, DeviceKind::CPU).await.unwrap();
        assert!(registry.is_model_loaded(&id));

        // Calling again should be idempotent
        registry.ensure_model_loaded(&id, DeviceKind::CPU).await.unwrap();
        assert!(registry.is_model_loaded(&id));
    }

    #[tokio::test]
    async fn test_unload_model() {
        let (registry, temp_dir) = setup_test_registry_with_engine().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let id = registry.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Load and unload
        registry.ensure_model_loaded(&id, DeviceKind::CPU).await.unwrap();
        assert!(registry.is_model_loaded(&id));

        registry.unload_model(&id).await.unwrap();
        assert!(!registry.is_model_loaded(&id));
    }

    #[tokio::test]
    async fn test_delete_model() {
        let (registry, temp_dir) = setup_test_registry().await;
        let model_path = create_temp_model_file(&temp_dir, "test.gguf");

        let id = registry.register_model(RegisterModelRequest {
            name: "test-model".to_string(),
            path: model_path,
            backend: "echo".to_string(),
            format: ModelFormat::GGUF,
            quantization: None,
            tags: vec![],
        }).await.unwrap();

        // Delete
        registry.delete_model(&id).await.unwrap();

        // Should not exist
        assert!(registry.get_model(&id).await.is_err());
    }

    #[tokio::test]
    async fn test_model_not_found() {
        let (registry, _temp_dir) = setup_test_registry().await;

        let result = registry.get_model("nonexistent").await;
        assert!(matches!(result, Err(ModelError::NotFound(_))));
    }
}
