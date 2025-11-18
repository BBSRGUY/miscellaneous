//! # Database Repositories
//!
//! CRUD operations for database entities.

use crate::entities::{Document, DocumentChunk, Job, Message, Model, Session};
use crate::{Result, StoreError};
use chrono::Utc;
use sqlx::SqlitePool;

/// Repository for Model entities.
pub struct ModelRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ModelRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new model.
    pub async fn create(&self, model: &Model) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO models (id, name, path, backend, format, size_bytes, quantization, tags, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&model.id)
        .bind(&model.name)
        .bind(&model.path)
        .bind(&model.backend)
        .bind(&model.format)
        .bind(model.size_bytes)
        .bind(&model.quantization)
        .bind(&model.tags)
        .bind(&model.created_at)
        .bind(&model.updated_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get a model by ID.
    pub async fn get(&self, id: &str) -> Result<Model> {
        let model = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
            .map_err(|_| StoreError::NotFound(format!("Model with id {} not found", id)))?;

        Ok(model)
    }

    /// Get a model by name.
    pub async fn get_by_name(&self, name: &str) -> Result<Model> {
        let model = sqlx::query_as::<_, Model>("SELECT * FROM models WHERE name = ?")
            .bind(name)
            .fetch_one(self.pool)
            .await
            .map_err(|_| StoreError::NotFound(format!("Model with name {} not found", name)))?;

        Ok(model)
    }

    /// List all models.
    pub async fn list(&self) -> Result<Vec<Model>> {
        let models = sqlx::query_as::<_, Model>("SELECT * FROM models ORDER BY created_at DESC")
            .fetch_all(self.pool)
            .await?;

        Ok(models)
    }

    /// Update a model.
    pub async fn update(&self, model: &Model) -> Result<()> {
        let updated_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE models
            SET name = ?, path = ?, backend = ?, format = ?, size_bytes = ?,
                quantization = ?, tags = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&model.name)
        .bind(&model.path)
        .bind(&model.backend)
        .bind(&model.format)
        .bind(model.size_bytes)
        .bind(&model.quantization)
        .bind(&model.tags)
        .bind(&updated_at)
        .bind(&model.id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Delete a model.
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM models WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

/// Repository for Session entities.
pub struct SessionRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new session.
    pub async fn create(&self, session: &Session) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, name, model_id, created_at, updated_at, config_json)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.name)
        .bind(&session.model_id)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .bind(&session.config_json)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get a session by ID.
    pub async fn get(&self, id: &str) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
            .map_err(|_| StoreError::NotFound(format!("Session with id {} not found", id)))?;

        Ok(session)
    }

    /// List all sessions.
    pub async fn list(&self) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>("SELECT * FROM sessions ORDER BY created_at DESC")
            .fetch_all(self.pool)
            .await?;

        Ok(sessions)
    }

    /// Update a session.
    pub async fn update(&self, session: &Session) -> Result<()> {
        let updated_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE sessions
            SET name = ?, model_id = ?, config_json = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&session.name)
        .bind(&session.model_id)
        .bind(&session.config_json)
        .bind(&updated_at)
        .bind(&session.id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Delete a session.
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

/// Repository for Message entities.
pub struct MessageRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> MessageRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new message.
    pub async fn create(&self, message: &Message) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO messages (id, session_id, role, content, meta_json, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(&message.session_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(&message.meta_json)
        .bind(&message.created_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get messages for a session.
    pub async fn get_by_session(&self, session_id: &str) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE session_id = ? ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await?;

        Ok(messages)
    }

    /// Delete a message.
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

/// Repository for Job entities.
pub struct JobRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> JobRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new job.
    pub async fn create(&self, job: &Job) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO jobs (id, kind, status, progress, created_at, updated_at, started_at, completed_at, config_json, logs_path, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&job.id)
        .bind(&job.kind)
        .bind(&job.status)
        .bind(job.progress)
        .bind(&job.created_at)
        .bind(&job.updated_at)
        .bind(&job.started_at)
        .bind(&job.completed_at)
        .bind(&job.config_json)
        .bind(&job.logs_path)
        .bind(&job.error_message)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get a job by ID.
    pub async fn get(&self, id: &str) -> Result<Job> {
        let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
            .map_err(|_| StoreError::NotFound(format!("Job with id {} not found", id)))?;

        Ok(job)
    }

    /// List all jobs.
    pub async fn list(&self) -> Result<Vec<Job>> {
        let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY created_at DESC")
            .fetch_all(self.pool)
            .await?;

        Ok(jobs)
    }

    /// List jobs by status.
    pub async fn list_by_status(&self, status: &str) -> Result<Vec<Job>> {
        let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE status = ? ORDER BY created_at DESC")
            .bind(status)
            .fetch_all(self.pool)
            .await?;

        Ok(jobs)
    }

    /// Update a job.
    pub async fn update(&self, job: &Job) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?, progress = ?, updated_at = ?, started_at = ?, completed_at = ?,
                config_json = ?, logs_path = ?, error_message = ?
            WHERE id = ?
            "#,
        )
        .bind(&job.status)
        .bind(job.progress)
        .bind(&job.updated_at)
        .bind(&job.started_at)
        .bind(&job.completed_at)
        .bind(&job.config_json)
        .bind(&job.logs_path)
        .bind(&job.error_message)
        .bind(&job.id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Delete a job.
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

/// Repository for Document entities.
pub struct DocumentRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> DocumentRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new document.
    pub async fn create(&self, document: &Document) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO documents (id, source, path, content, meta_json, created_at, updated_at, indexed_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&document.id)
        .bind(&document.source)
        .bind(&document.path)
        .bind(&document.content)
        .bind(&document.meta_json)
        .bind(&document.created_at)
        .bind(&document.updated_at)
        .bind(&document.indexed_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get a document by ID.
    pub async fn get(&self, id: &str) -> Result<Document> {
        let document = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
            .map_err(|_| StoreError::NotFound(format!("Document with id {} not found", id)))?;

        Ok(document)
    }

    /// List all documents.
    pub async fn list(&self) -> Result<Vec<Document>> {
        let documents = sqlx::query_as::<_, Document>("SELECT * FROM documents ORDER BY created_at DESC")
            .fetch_all(self.pool)
            .await?;

        Ok(documents)
    }

    /// Update a document.
    pub async fn update(&self, document: &Document) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE documents
            SET source = ?, path = ?, content = ?, meta_json = ?, updated_at = ?, indexed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&document.source)
        .bind(&document.path)
        .bind(&document.content)
        .bind(&document.meta_json)
        .bind(&document.updated_at)
        .bind(&document.indexed_at)
        .bind(&document.id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Delete a document.
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}

/// Repository for DocumentChunk entities (RAG support).
pub struct DocumentChunkRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> DocumentChunkRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new document chunk.
    pub async fn create(&self, chunk: &DocumentChunk) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO document_chunks (id, document_id, chunk_index, content, embedding, metadata, token_count, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&chunk.id)
        .bind(&chunk.document_id)
        .bind(chunk.chunk_index)
        .bind(&chunk.content)
        .bind(&chunk.embedding)
        .bind(&chunk.metadata)
        .bind(chunk.token_count)
        .bind(&chunk.created_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get a chunk by ID.
    pub async fn get(&self, id: &str) -> Result<DocumentChunk> {
        let chunk = sqlx::query_as::<_, DocumentChunk>("SELECT * FROM document_chunks WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool)
            .await
            .map_err(|_| StoreError::NotFound(format!("Document chunk with id {} not found", id)))?;

        Ok(chunk)
    }

    /// List all chunks for a document.
    pub async fn list_by_document(&self, document_id: &str) -> Result<Vec<DocumentChunk>> {
        let chunks = sqlx::query_as::<_, DocumentChunk>(
            "SELECT * FROM document_chunks WHERE document_id = ? ORDER BY chunk_index ASC",
        )
        .bind(document_id)
        .fetch_all(self.pool)
        .await?;

        Ok(chunks)
    }

    /// Search for similar chunks using cosine similarity.
    ///
    /// This performs an in-memory similarity search. For large datasets,
    /// consider using a dedicated vector database or approximate nearest neighbor index.
    pub async fn search_by_similarity(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(DocumentChunk, f32)>> {
        // Fetch all chunks with embeddings
        let chunks = sqlx::query_as::<_, DocumentChunk>(
            "SELECT * FROM document_chunks WHERE embedding IS NOT NULL",
        )
        .fetch_all(self.pool)
        .await?;

        // Create a temporary chunk with the query embedding for similarity calculation
        let mut query_chunk = DocumentChunk {
            id: String::new(),
            document_id: String::new(),
            chunk_index: 0,
            content: String::new(),
            embedding: None,
            metadata: None,
            token_count: None,
            created_at: String::new(),
        };
        query_chunk.set_embedding(query_embedding);

        // Calculate similarities
        let mut similarities: Vec<(DocumentChunk, f32)> = chunks
            .into_iter()
            .filter_map(|chunk| {
                chunk
                    .cosine_similarity(&query_chunk)
                    .map(|similarity| (chunk, similarity))
            })
            .collect();

        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        similarities.truncate(limit);

        Ok(similarities)
    }

    /// Count chunks for a document.
    pub async fn count_by_document(&self, document_id: &str) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM document_chunks WHERE document_id = ?",
        )
        .bind(document_id)
        .fetch_one(self.pool)
        .await?;

        Ok(result)
    }

    /// Delete all chunks for a document.
    pub async fn delete_by_document(&self, document_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM document_chunks WHERE document_id = ?")
            .bind(document_id)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    /// Delete a specific chunk.
    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM document_chunks WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}
