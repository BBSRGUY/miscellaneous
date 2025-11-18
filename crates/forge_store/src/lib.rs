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

    #[tokio::test]
    async fn test_document_crud() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.documents();

        // Create
        let mut doc = Document::new("test-doc.txt".to_string());
        doc.content = Some("This is a test document".to_string());
        doc.path = Some("/path/to/doc.txt".to_string());

        repo.create(&doc).await.unwrap();

        // Read
        let retrieved = repo.get(&doc.id).await.unwrap();
        assert_eq!(retrieved.source, "test-doc.txt");
        assert_eq!(retrieved.content, Some("This is a test document".to_string()));
        assert!(retrieved.indexed_at.is_none());

        // Mark as indexed
        let mut updated = retrieved.clone();
        updated.mark_indexed();
        repo.update(&updated).await.unwrap();

        let indexed = repo.get(&doc.id).await.unwrap();
        assert!(indexed.indexed_at.is_some());

        // List
        let docs = repo.list().await.unwrap();
        assert_eq!(docs.len(), 1);

        // Delete
        repo.delete(&doc.id).await.unwrap();
        let result = repo.get(&doc.id).await;
        assert!(result.is_err());

        store.close().await;
    }

    #[tokio::test]
    async fn test_document_chunks_and_embeddings() {
        let store = Store::new_in_memory().await.unwrap();

        // Create a document
        let doc = Document::new("rag-test.txt".to_string());
        store.documents().create(&doc).await.unwrap();

        // Create chunks with embeddings
        let mut chunk1 = DocumentChunk::new(doc.id.clone(), 0, "First chunk".to_string());
        chunk1.set_embedding(vec![0.1, 0.2, 0.3, 0.4]);
        chunk1.token_count = Some(2);

        let mut chunk2 = DocumentChunk::new(doc.id.clone(), 1, "Second chunk".to_string());
        chunk2.set_embedding(vec![0.5, 0.6, 0.7, 0.8]);
        chunk2.token_count = Some(2);

        let chunk_repo = store.document_chunks();
        chunk_repo.create(&chunk1).await.unwrap();
        chunk_repo.create(&chunk2).await.unwrap();

        // List chunks for document
        let chunks = chunk_repo.list_by_document(&doc.id).await.unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);

        // Count chunks
        let count = chunk_repo.count_by_document(&doc.id).await.unwrap();
        assert_eq!(count, 2);

        // Verify embeddings are stored and retrieved correctly
        let retrieved_chunk = chunk_repo.get(&chunk1.id).await.unwrap();
        let embedding = retrieved_chunk.get_embedding().unwrap();
        assert_eq!(embedding, vec![0.1, 0.2, 0.3, 0.4]);

        // Test cosine similarity
        let similarity = chunk1.cosine_similarity(&chunk2).unwrap();
        assert!(similarity > 0.0 && similarity <= 1.0);

        // Test similarity search
        let query_embedding = vec![0.2, 0.3, 0.4, 0.5];
        let results = chunk_repo
            .search_by_similarity(query_embedding, 2)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].1 >= results[1].1); // First result should have higher similarity

        // Delete chunks
        chunk_repo.delete_by_document(&doc.id).await.unwrap();
        let chunks = chunk_repo.list_by_document(&doc.id).await.unwrap();
        assert_eq!(chunks.len(), 0);

        store.close().await;
    }

    #[tokio::test]
    async fn test_model_get_by_name() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.models();

        let model = Model::new(
            "unique-model".to_string(),
            "echo".to_string(),
            "gguf".to_string(),
        );
        repo.create(&model).await.unwrap();

        // Get by name
        let retrieved = repo.get_by_name("unique-model").await.unwrap();
        assert_eq!(retrieved.id, model.id);

        // Test not found
        let result = repo.get_by_name("non-existent").await;
        assert!(result.is_err());

        store.close().await;
    }

    #[tokio::test]
    async fn test_job_status_filtering() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.jobs();

        // Create jobs with different statuses
        let mut job1 = Job::new("training".to_string());
        job1.start();
        repo.create(&job1).await.unwrap();

        let mut job2 = Job::new("inference".to_string());
        repo.create(&job2).await.unwrap();

        let mut job3 = Job::new("training".to_string());
        job3.start();
        job3.complete();
        repo.create(&job3).await.unwrap();

        // List all jobs
        let all_jobs = repo.list().await.unwrap();
        assert_eq!(all_jobs.len(), 3);

        // Filter by status
        let running = repo.list_by_status("running").await.unwrap();
        assert_eq!(running.len(), 1);

        let queued = repo.list_by_status("queued").await.unwrap();
        assert_eq!(queued.len(), 1);

        let completed = repo.list_by_status("completed").await.unwrap();
        assert_eq!(completed.len(), 1);

        store.close().await;
    }

    #[tokio::test]
    async fn test_job_failure() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.jobs();

        let mut job = Job::new("training".to_string());
        job.start();
        repo.create(&job).await.unwrap();

        // Fail the job
        job.fail("Out of memory".to_string());
        repo.update(&job).await.unwrap();

        let retrieved = repo.get(&job.id).await.unwrap();
        assert_eq!(retrieved.status, "failed");
        assert_eq!(retrieved.error_message, Some("Out of memory".to_string()));
        assert!(retrieved.completed_at.is_some());

        store.close().await;
    }

    #[tokio::test]
    async fn test_session_list_ordering() {
        let store = Store::new_in_memory().await.unwrap();
        let repo = store.sessions();

        // Create multiple sessions
        let session1 = Session::new("First".to_string(), None);
        let session2 = Session::new("Second".to_string(), None);
        let session3 = Session::new("Third".to_string(), None);

        repo.create(&session1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        repo.create(&session2).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        repo.create(&session3).await.unwrap();

        // List should be ordered by created_at DESC (newest first)
        let sessions = repo.list().await.unwrap();
        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0].name, "Third");
        assert_eq!(sessions[2].name, "First");

        store.close().await;
    }

    #[tokio::test]
    async fn test_repository_not_found_errors() {
        let store = Store::new_in_memory().await.unwrap();

        // Test model not found
        let result = store.models().get("non-existent-id").await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        // Test session not found
        let result = store.sessions().get("non-existent-id").await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        // Test job not found
        let result = store.jobs().get("non-existent-id").await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        // Test document not found
        let result = store.documents().get("non-existent-id").await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        // Test document chunk not found
        let result = store.document_chunks().get("non-existent-id").await;
        assert!(matches!(result, Err(StoreError::NotFound(_))));

        store.close().await;
    }
}
