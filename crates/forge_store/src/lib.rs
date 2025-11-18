//! # Forge Store
//!
//! Database, migrations, blob storage, and repositories.
//!
//! This crate handles persistent storage for Forge, including SQLite database
//! management, migrations, and blob storage for models and datasets.

pub mod blob;
pub mod entities;
pub mod repositories;

use blob::BlobStorage;
use repositories::{
    DocumentChunkRepository, DocumentRepository, JobRepository, MessageRepository, ModelRepository,
    SessionRepository,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use tracing::{debug, info, log::LevelFilter};

pub use blob::BlobCategory;
pub use entities::{Document, DocumentChunk, Job, Message, Model, Session};

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// Database connection pool and storage manager.
pub struct Store {
    pool: SqlitePool,
    blob_storage: BlobStorage,
}

impl Store {
    /// Create a new store with an in-memory database.
    pub async fn new_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Warn, std::time::Duration::from_secs(1));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| StoreError::Migration(e.to_string()))?;

        // Create temporary blob storage for in-memory database
        let temp_dir = std::env::temp_dir().join(format!("forge-blob-{}", uuid::Uuid::new_v4()));
        let blob_storage = BlobStorage::new(&temp_dir)?;

        info!("Store initialized successfully (in-memory)");

        Ok(Self { pool, blob_storage })
    }

    /// Create a new store with a file-based database.
    pub async fn new_with_file<P: AsRef<Path>>(
        db_path: P,
        blob_storage_root: P,
    ) -> Result<Self> {
        let db_path = db_path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let db_url = format!("sqlite:{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Warn, std::time::Duration::from_secs(1));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        info!("Connected to database: {}", db_path.display());
        info!("Running database migrations...");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| StoreError::Migration(e.to_string()))?;

        debug!("Migrations completed successfully");

        let blob_storage = BlobStorage::new(&blob_storage_root)?;

        info!("Store initialized successfully");
        info!("Database: {}", db_path.display());
        info!("Blob storage: {}", blob_storage.storage_root().display());

        Ok(Self { pool, blob_storage })
    }

    /// Create a new store from configuration paths.
    pub async fn from_config<P: AsRef<Path>>(
        db_path: P,
        storage_root: P,
    ) -> Result<Self> {
        Self::new_with_file(db_path, storage_root).await
    }

    /// Get the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the blob storage manager.
    pub fn blob_storage(&self) -> &BlobStorage {
        &self.blob_storage
    }

    /// Get a model repository.
    pub fn models(&self) -> ModelRepository<'_> {
        ModelRepository::new(&self.pool)
    }

    /// Get a session repository.
    pub fn sessions(&self) -> SessionRepository<'_> {
        SessionRepository::new(&self.pool)
    }

    /// Get a message repository.
    pub fn messages(&self) -> MessageRepository<'_> {
        MessageRepository::new(&self.pool)
    }

    /// Get a job repository.
    pub fn jobs(&self) -> JobRepository<'_> {
        JobRepository::new(&self.pool)
    }

    /// Get a document repository.
    pub fn documents(&self) -> DocumentRepository<'_> {
        DocumentRepository::new(&self.pool)
    }

    /// Get a document chunk repository.
    pub fn document_chunks(&self) -> DocumentChunkRepository<'_> {
        DocumentChunkRepository::new(&self.pool)
    }

    /// Close the store and release all connections.
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_in_memory_store() {
        let store = Store::new_in_memory().await.unwrap();
        assert!(!store.pool().is_closed());
        store.close().await;
    }

    #[tokio::test]
    async fn test_create_file_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let blob_path = temp_dir.path().join("blobs");

        let store = Store::new_with_file(&db_path, &blob_path)
            .await
            .unwrap();

        assert!(!store.pool().is_closed());
        assert!(db_path.exists());
        assert!(blob_path.exists());

        store.close().await;
    }

    #[tokio::test]
    async fn test_migrations_run() {
        let store = Store::new_in_memory().await.unwrap();

        // Verify tables exist by attempting to query them
        let result = sqlx::query("SELECT COUNT(*) FROM models")
            .fetch_one(store.pool())
            .await;

        assert!(result.is_ok());

        let result = sqlx::query("SELECT COUNT(*) FROM sessions")
            .fetch_one(store.pool())
            .await;

        assert!(result.is_ok());

        let result = sqlx::query("SELECT COUNT(*) FROM messages")
            .fetch_one(store.pool())
            .await;

        assert!(result.is_ok());

        let result = sqlx::query("SELECT COUNT(*) FROM jobs")
            .fetch_one(store.pool())
            .await;

        assert!(result.is_ok());

        let result = sqlx::query("SELECT COUNT(*) FROM documents")
            .fetch_one(store.pool())
            .await;

        assert!(result.is_ok());

        store.close().await;
    }

    #[tokio::test]
    async fn test_model_crud() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.models();

        // Create
        let mut model = Model::new(
            "test-model".to_string(),
            "echo".to_string(),
            "gguf".to_string(),
        );
        model.size_bytes = Some(1024);
        model.set_tags(vec!["test".to_string(), "demo".to_string()]);

        repo.create(&model).await.unwrap();

        // Read
        let retrieved = repo.get(&model.id).await.unwrap();
        assert_eq!(retrieved.name, "test-model");
        assert_eq!(retrieved.backend, "echo");
        assert_eq!(retrieved.parse_tags(), vec!["test", "demo"]);

        // Update
        let mut updated = retrieved.clone();
        updated.backend = "llama".to_string();
        repo.update(&updated).await.unwrap();

        let retrieved = repo.get(&model.id).await.unwrap();
        assert_eq!(retrieved.backend, "llama");

        // List
        let models = repo.list().await.unwrap();
        assert_eq!(models.len(), 1);

        // Delete
        repo.delete(&model.id).await.unwrap();

        let result = repo.get(&model.id).await;
        assert!(result.is_err());

        store.close().await;
    }

    #[tokio::test]
    async fn test_session_with_messages() {
        let store = Store::new_in_memory().await.unwrap();

        // Create a session
        let session = Session::new("Test Session".to_string(), None);
        store.sessions().create(&session).await.unwrap();

        // Add messages
        let msg1 = Message::new(session.id.clone(), "user".to_string(), "Hello".to_string());
        let msg2 = Message::new(
            session.id.clone(),
            "assistant".to_string(),
            "Hi there!".to_string(),
        );

        store.messages().create(&msg1).await.unwrap();
        store.messages().create(&msg2).await.unwrap();

        // Retrieve messages
        let messages = store.messages().get_by_session(&session.id).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi there!");

        // Delete session (should cascade to messages)
        store.sessions().delete(&session.id).await.unwrap();

        let messages = store.messages().get_by_session(&session.id).await.unwrap();
        assert_eq!(messages.len(), 0);

        store.close().await;
    }

    #[tokio::test]
    async fn test_job_lifecycle() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.jobs();

        // Create a job
        let mut job = Job::new("training".to_string());
        job.config_json = Some(r#"{"epochs": 10}"#.to_string());

        repo.create(&job).await.unwrap();

        // Start the job
        job.start();
        repo.update(&job).await.unwrap();

        let retrieved = repo.get(&job.id).await.unwrap();
        assert_eq!(retrieved.status, "running");
        assert!(retrieved.started_at.is_some());

        // Complete the job
        job.complete();
        repo.update(&job).await.unwrap();

        let retrieved = repo.get(&job.id).await.unwrap();
        assert_eq!(retrieved.status, "completed");
        assert_eq!(retrieved.progress, 1.0);
        assert!(retrieved.completed_at.is_some());

        store.close().await;
    }

    #[tokio::test]
    async fn test_blob_storage_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let blob_path = temp_dir.path().join("blobs");

        let store = Store::new_with_file(&db_path, &blob_path)
            .await
            .unwrap();

        // Store a model blob
        let model_data = b"fake model weights";
        let blob_path = store
            .blob_storage()
            .store_blob_named(BlobCategory::Models, "model.bin", model_data)
            .unwrap();

        // Create model record with blob path
        let mut model = Model::new(
            "blob-model".to_string(),
            "custom".to_string(),
            "bin".to_string(),
        );
        model.path = Some(blob_path.clone());
        model.size_bytes = Some(model_data.len() as i64);

        store.models().create(&model).await.unwrap();

        // Retrieve and verify
        let retrieved_model = store.models().get(&model.id).await.unwrap();
        assert_eq!(retrieved_model.path, Some(blob_path.clone()));

        let retrieved_data = store
            .blob_storage()
            .get_blob(&blob_path)
            .unwrap();
        assert_eq!(retrieved_data, model_data);

        store.close().await;
    }
}
