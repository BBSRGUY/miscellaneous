//! # Database Entities
//!
//! Struct definitions for database entities.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Model entity representing an LLM model.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub backend: String,
    pub format: String,
    pub size_bytes: Option<i64>,
    pub quantization: Option<String>,
    pub tags: Option<String>, // JSON array
    pub created_at: String,
    pub updated_at: String,
}

impl Model {
    /// Create a new model entity.
    pub fn new(name: String, backend: String, format: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            path: None,
            backend,
            format,
            size_bytes: None,
            quantization: None,
            tags: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Parse tags from JSON.
    pub fn parse_tags(&self) -> Vec<String> {
        if let Some(tags_json) = &self.tags {
            serde_json::from_str(tags_json).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Set tags from a vector.
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = Some(serde_json::to_string(&tags).unwrap());
    }
}

/// Session entity representing a chat session.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub model_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub config_json: Option<String>,
}

impl Session {
    /// Create a new session entity.
    pub fn new(name: String, model_id: Option<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            model_id,
            created_at: now.clone(),
            updated_at: now,
            config_json: None,
        }
    }
}

/// Message entity representing a chat message.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String, // 'user', 'assistant', 'system'
    pub content: String,
    pub meta_json: Option<String>,
    pub created_at: String,
}

impl Message {
    /// Create a new message entity.
    pub fn new(session_id: String, role: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id,
            role,
            content,
            meta_json: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Job entity representing a background task.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Job {
    pub id: String,
    pub kind: String, // 'inference', 'training', 'embedding'
    pub status: String, // 'queued', 'running', 'completed', 'failed', 'cancelled'
    pub progress: f64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub config_json: Option<String>,
    pub logs_path: Option<String>,
    pub error_message: Option<String>,
}

impl Job {
    /// Create a new job entity.
    pub fn new(kind: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            status: "queued".to_string(),
            progress: 0.0,
            created_at: now.clone(),
            updated_at: now,
            started_at: None,
            completed_at: None,
            config_json: None,
            logs_path: None,
            error_message: None,
        }
    }

    /// Mark job as started.
    pub fn start(&mut self) {
        self.status = "running".to_string();
        self.started_at = Some(Utc::now().to_rfc3339());
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Mark job as completed.
    pub fn complete(&mut self) {
        self.status = "completed".to_string();
        self.progress = 1.0;
        self.completed_at = Some(Utc::now().to_rfc3339());
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Mark job as failed.
    pub fn fail(&mut self, error: String) {
        self.status = "failed".to_string();
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now().to_rfc3339());
        self.updated_at = Utc::now().to_rfc3339();
    }
}

/// Document entity for RAG.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Document {
    pub id: String,
    pub source: String,
    pub path: Option<String>,
    pub content: Option<String>,
    pub meta_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub indexed_at: Option<String>,
}

impl Document {
    /// Create a new document entity.
    pub fn new(source: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            path: None,
            content: None,
            meta_json: None,
            created_at: now.clone(),
            updated_at: now,
            indexed_at: None,
        }
    }

    /// Mark document as indexed.
    pub fn mark_indexed(&mut self) {
        self.indexed_at = Some(Utc::now().to_rfc3339());
        self.updated_at = Utc::now().to_rfc3339();
    }
}

/// Document chunk entity for RAG vector storage.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: i32,
    pub content: String,
    pub embedding: Option<Vec<u8>>, // BLOB storage for vector embeddings
    pub metadata: Option<String>, // JSON metadata
    pub token_count: Option<i32>,
    pub created_at: String,
}

impl DocumentChunk {
    /// Create a new document chunk.
    pub fn new(document_id: String, chunk_index: i32, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            document_id,
            chunk_index,
            content,
            embedding: None,
            metadata: None,
            token_count: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    /// Set the embedding vector.
    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        // Convert f32 vector to bytes
        self.embedding = Some(embedding.iter().flat_map(|f| f.to_le_bytes()).collect());
    }

    /// Get the embedding as f32 vector.
    pub fn get_embedding(&self) -> Option<Vec<f32>> {
        self.embedding.as_ref().map(|bytes| {
            bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect()
        })
    }

    /// Calculate cosine similarity with another chunk.
    pub fn cosine_similarity(&self, other: &DocumentChunk) -> Option<f32> {
        let a = self.get_embedding()?;
        let b = other.get_embedding()?;

        if a.len() != b.len() {
            return None;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return None;
        }

        Some(dot_product / (norm_a * norm_b))
    }
}
