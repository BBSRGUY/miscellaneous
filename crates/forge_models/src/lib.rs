//! # Forge Models
//!
//! Model registry, loading, and metadata management.
//!
//! This crate handles the catalog of available models, their installation,
//! loading state, and metadata.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during model operations.
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Model already exists: {0}")]
    AlreadyExists(String),

    #[error("Model is currently loaded")]
    AlreadyLoaded,

    #[error("Invalid model metadata: {0}")]
    InvalidMetadata(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ModelError>;

/// Represents a model's metadata in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub architecture: String,
    pub parameters: u64,
    pub context_length: usize,
    pub file_path: Option<PathBuf>,
    pub file_size: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// Current state of a model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelState {
    /// Model is registered but not installed
    Registered,
    /// Model files are being downloaded
    Downloading,
    /// Model is installed but not loaded
    Installed,
    /// Model is loaded and ready for inference
    Loaded,
    /// Model encountered an error
    Error(String),
}

/// A model entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub metadata: ModelMetadata,
    pub state: ModelState,
}

/// In-memory model registry.
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<Uuid, ModelEntry>>>,
}

impl ModelRegistry {
    /// Create a new empty model registry.
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new model.
    pub fn register(&self, metadata: ModelMetadata) -> Result<Uuid> {
        let mut models = self.models.write();

        // Check if model with this name already exists
        if models.values().any(|entry| entry.metadata.name == metadata.name) {
            return Err(ModelError::AlreadyExists(metadata.name.clone()));
        }

        let id = metadata.id;
        let entry = ModelEntry {
            metadata,
            state: ModelState::Registered,
        };

        models.insert(id, entry);
        Ok(id)
    }

    /// Get a model by ID.
    pub fn get(&self, id: &Uuid) -> Result<ModelEntry> {
        let models = self.models.read();
        models
            .get(id)
            .cloned()
            .ok_or_else(|| ModelError::NotFound(id.to_string()))
    }

    /// Get a model by name.
    pub fn get_by_name(&self, name: &str) -> Result<ModelEntry> {
        let models = self.models.read();
        models
            .values()
            .find(|entry| entry.metadata.name == name)
            .cloned()
            .ok_or_else(|| ModelError::NotFound(name.to_string()))
    }

    /// List all models.
    pub fn list(&self) -> Vec<ModelEntry> {
        let models = self.models.read();
        models.values().cloned().collect()
    }

    /// Update model state.
    pub fn update_state(&self, id: &Uuid, state: ModelState) -> Result<()> {
        let mut models = self.models.write();
        let entry = models
            .get_mut(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))?;

        entry.state = state;
        Ok(())
    }

    /// Remove a model from the registry.
    pub fn unregister(&self, id: &Uuid) -> Result<ModelEntry> {
        let mut models = self.models.write();
        models
            .remove(id)
            .ok_or_else(|| ModelError::NotFound(id.to_string()))
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> ModelMetadata {
        ModelMetadata {
            id: Uuid::new_v4(),
            name: "test-model".to_string(),
            description: "A test model".to_string(),
            architecture: "transformer".to_string(),
            parameters: 7_000_000_000,
            context_length: 2048,
            file_path: None,
            file_size: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_register_model() {
        let registry = ModelRegistry::new();
        let metadata = create_test_metadata();
        let id = metadata.id;

        let result = registry.register(metadata);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);
    }

    #[test]
    fn test_get_model() {
        let registry = ModelRegistry::new();
        let metadata = create_test_metadata();
        let id = metadata.id;

        registry.register(metadata.clone()).unwrap();
        let entry = registry.get(&id).unwrap();
        assert_eq!(entry.metadata.name, metadata.name);
    }

    #[test]
    fn test_list_models() {
        let registry = ModelRegistry::new();
        let metadata1 = create_test_metadata();
        let mut metadata2 = create_test_metadata();
        metadata2.id = Uuid::new_v4();
        metadata2.name = "test-model-2".to_string();

        registry.register(metadata1).unwrap();
        registry.register(metadata2).unwrap();

        let models = registry.list();
        assert_eq!(models.len(), 2);
    }

    #[test]
    fn test_update_state() {
        let registry = ModelRegistry::new();
        let metadata = create_test_metadata();
        let id = metadata.id;

        registry.register(metadata).unwrap();
        registry.update_state(&id, ModelState::Loaded).unwrap();

        let entry = registry.get(&id).unwrap();
        assert_eq!(entry.state, ModelState::Loaded);
    }
}
