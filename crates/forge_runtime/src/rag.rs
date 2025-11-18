//! # RAG (Retrieval-Augmented Generation) Support
//!
//! Document ingestion, chunking, embedding, and retrieval for RAG pipelines.

use forge_engine::Engine;
use forge_store::{Document, DocumentChunk, Store};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, warn};

/// Errors that can occur during RAG operations.
#[derive(Debug, Error)]
pub enum RagError {
    #[error("Storage error: {0}")]
    Storage(#[from] forge_store::StoreError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Engine error: {0}")]
    Engine(#[from] forge_engine::EngineError),

    #[error("Invalid document format: {0}")]
    InvalidFormat(String),

    #[error("Embedding model not configured")]
    NoEmbeddingModel,

    #[error("Document not found: {0}")]
    DocumentNotFound(String),
}

pub type Result<T> = std::result::Result<T, RagError>;

/// Configuration for document chunking.
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Target chunk size in characters
    pub chunk_size: usize,
    /// Overlap between chunks in characters
    pub chunk_overlap: usize,
    /// Minimum chunk size (discard smaller chunks)
    pub min_chunk_size: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 128,
            min_chunk_size: 50,
        }
    }
}

/// Document text chunker.
pub struct TextChunker {
    config: ChunkConfig,
}

impl TextChunker {
    /// Create a new text chunker with configuration.
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// Create a text chunker with default configuration.
    pub fn default() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }

    /// Split text into chunks with overlap.
    pub fn chunk(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_len = chars.len();

        if total_len == 0 {
            return chunks;
        }

        let mut start = 0;

        while start < total_len {
            let end = (start + self.config.chunk_size).min(total_len);
            let chunk: String = chars[start..end].iter().collect();

            // Only add chunk if it meets minimum size
            if chunk.len() >= self.config.min_chunk_size {
                chunks.push(chunk);
            }

            // Move forward by chunk_size - overlap
            let step = self.config.chunk_size.saturating_sub(self.config.chunk_overlap);
            if step == 0 {
                break; // Prevent infinite loop
            }
            start += step;

            // If we've covered everything, break
            if end >= total_len {
                break;
            }
        }

        chunks
    }

    /// Split text into chunks with smart sentence boundaries.
    ///
    /// This attempts to split on sentence boundaries when possible.
    pub fn chunk_smart(&self, text: &str) -> Vec<String> {
        let sentences = self.split_sentences(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for sentence in sentences {
            // If adding this sentence would exceed chunk size
            if current_chunk.len() + sentence.len() > self.config.chunk_size {
                // Save current chunk if it meets minimum size
                if current_chunk.len() >= self.config.min_chunk_size {
                    chunks.push(current_chunk.clone());
                }

                // Start new chunk with overlap from previous
                if self.config.chunk_overlap > 0 && current_chunk.len() > self.config.chunk_overlap {
                    let overlap_start = current_chunk.len() - self.config.chunk_overlap;
                    current_chunk = current_chunk[overlap_start..].to_string();
                } else {
                    current_chunk.clear();
                }
            }

            current_chunk.push_str(sentence);
            current_chunk.push(' ');
        }

        // Add final chunk
        if current_chunk.len() >= self.config.min_chunk_size {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Simple sentence splitter.
    fn split_sentences<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut sentences = Vec::new();
        let mut start = 0;

        for (i, c) in text.char_indices() {
            if c == '.' || c == '!' || c == '?' {
                // Check if next char is whitespace (end of sentence)
                if let Some(next_char) = text[i + 1..].chars().next() {
                    if next_char.is_whitespace() {
                        sentences.push(&text[start..=i]);
                        start = i + 1;
                    }
                } else {
                    // End of text
                    sentences.push(&text[start..=i]);
                }
            }
        }

        // Add remaining text
        if start < text.len() {
            sentences.push(&text[start..]);
        }

        sentences
    }
}

/// RAG service for document ingestion and retrieval.
pub struct RagService {
    store: Arc<Store>,
    engine: Arc<Engine>,
    chunker: TextChunker,
    embedding_model_id: Option<String>,
}

impl RagService {
    /// Create a new RAG service.
    pub fn new(store: Arc<Store>, engine: Arc<Engine>) -> Self {
        Self {
            store,
            engine,
            chunker: TextChunker::default(),
            embedding_model_id: None,
        }
    }

    /// Create a new RAG service with custom chunk configuration.
    pub fn with_config(store: Arc<Store>, engine: Arc<Engine>, chunk_config: ChunkConfig) -> Self {
        Self {
            store,
            engine,
            chunker: TextChunker::new(chunk_config),
            embedding_model_id: None,
        }
    }

    /// Set the embedding model to use for encoding documents.
    pub fn set_embedding_model(&mut self, model_id: String) {
        self.embedding_model_id = Some(model_id);
    }

    /// Ingest a document from a file path.
    pub async fn ingest_file(&self, file_path: &Path) -> Result<String> {
        info!("Ingesting document from: {}", file_path.display());

        // Read file content
        let content = self.read_file(file_path).await?;

        // Create document entity
        let mut document = Document::new(file_path.display().to_string());
        document.path = Some(file_path.display().to_string());
        document.content = Some(content.clone());

        // Save document
        self.store.documents().create(&document).await?;
        info!("Created document: {}", document.id);

        // Chunk the document
        let chunks = self.chunker.chunk_smart(&content);
        info!("Split document into {} chunks", chunks.len());

        // Create and store chunks
        for (index, chunk_text) in chunks.iter().enumerate() {
            let mut chunk = DocumentChunk::new(document.id.clone(), index as i32, chunk_text.clone());

            // Generate embedding if model is configured
            if let Some(model_id) = &self.embedding_model_id {
                match self.generate_embedding(model_id, chunk_text).await {
                    Ok(embedding) => {
                        chunk.set_embedding(embedding);
                    }
                    Err(e) => {
                        warn!("Failed to generate embedding for chunk {}: {}", index, e);
                    }
                }
            }

            self.store.document_chunks().create(&chunk).await?;
        }

        // Mark document as indexed
        document.mark_indexed();
        self.store.documents().update(&document).await?;

        info!("Successfully ingested document: {}", document.id);
        Ok(document.id)
    }

    /// Read and extract text from a file.
    async fn read_file(&self, path: &Path) -> Result<String> {
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" | "md" | "markdown" => {
                // Plain text files
                Ok(fs::read_to_string(path).await?)
            }
            "pdf" => {
                // PDF extraction (simplified - would need a PDF library in production)
                warn!("PDF extraction not fully implemented, treating as text");
                Ok(fs::read_to_string(path).await?)
            }
            _ => Err(RagError::InvalidFormat(format!(
                "Unsupported file format: {}",
                extension
            ))),
        }
    }

    /// Generate embedding for text using the configured embedding model.
    async fn generate_embedding(&self, _model_id: &str, text: &str) -> Result<Vec<f32>> {
        // For now, use a simple mock embedding
        // In production, this would call an actual embedding model
        debug!("Generating embedding for text (length: {})", text.len());

        // Mock embedding: create a simple hash-based embedding
        let embedding = self.mock_embedding(text);

        Ok(embedding)
    }

    /// Generate a mock embedding for demonstration.
    ///
    /// In production, this would be replaced with actual embedding model inference.
    fn mock_embedding(&self, text: &str) -> Vec<f32> {
        // Create a deterministic 384-dimensional embedding based on text
        let mut embedding = vec![0.0f32; 384];

        // Simple hash-based embedding for demonstration
        for (i, c) in text.chars().enumerate() {
            let idx = (c as usize + i) % 384;
            embedding[idx] += 1.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }

    /// Query for relevant document chunks.
    pub async fn query(&self, query_text: &str, limit: usize) -> Result<Vec<(DocumentChunk, f32)>> {
        info!("Querying RAG with: {} (limit: {})", query_text, limit);

        // Generate embedding for query
        let query_embedding = if let Some(model_id) = &self.embedding_model_id {
            self.generate_embedding(model_id, query_text).await?
        } else {
            // Use mock embedding if no model configured
            self.mock_embedding(query_text)
        };

        // Search for similar chunks
        let results = self
            .store
            .document_chunks()
            .search_by_similarity(query_embedding, limit)
            .await?;

        info!("Found {} relevant chunks", results.len());
        Ok(results)
    }

    /// Query and format results for LLM context.
    pub async fn query_for_context(&self, query_text: &str, limit: usize) -> Result<String> {
        let results = self.query(query_text, limit).await?;

        let mut context = String::new();
        context.push_str("# Retrieved Context\n\n");

        for (i, (chunk, similarity)) in results.iter().enumerate() {
            context.push_str(&format!(
                "## Document {} (relevance: {:.3})\n{}\n\n",
                i + 1,
                similarity,
                chunk.content
            ));
        }

        Ok(context)
    }

    /// List all documents.
    pub async fn list_documents(&self) -> Result<Vec<Document>> {
        Ok(self.store.documents().list().await?)
    }

    /// Get a document by ID.
    pub async fn get_document(&self, id: &str) -> Result<Document> {
        Ok(self.store.documents().get(id).await?)
    }

    /// Get chunks for a document.
    pub async fn get_document_chunks(&self, document_id: &str) -> Result<Vec<DocumentChunk>> {
        Ok(self.store.document_chunks().list_by_document(document_id).await?)
    }

    /// Delete a document and its chunks.
    pub async fn delete_document(&self, id: &str) -> Result<()> {
        // Delete chunks first (should cascade via foreign key, but explicit is safer)
        self.store.document_chunks().delete_by_document(id).await?;

        // Delete document
        self.store.documents().delete(id).await?;

        info!("Deleted document: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_chunker() {
        let chunker = TextChunker::new(ChunkConfig {
            chunk_size: 20,
            chunk_overlap: 5,
            min_chunk_size: 5,
        });

        let text = "This is a test document. It has multiple sentences. We want to chunk it properly.";
        let chunks = chunker.chunk(text);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.len() <= 20);
        }
    }

    #[test]
    fn test_smart_chunker() {
        let chunker = TextChunker::new(ChunkConfig {
            chunk_size: 50,
            chunk_overlap: 10,
            min_chunk_size: 10,
        });

        let text = "First sentence. Second sentence. Third sentence. Fourth sentence.";
        let chunks = chunker.chunk_smart(text);

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.len() >= 10);
        }
    }

    #[test]
    fn test_mock_embedding() {
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        let backend = Arc::new(forge_engine::EchoBackend::new());
        let engine = Arc::new(Engine::new(backend));
        let rag = RagService::new(store, engine);

        let embedding1 = rag.mock_embedding("test");
        let embedding2 = rag.mock_embedding("test");
        let embedding3 = rag.mock_embedding("different");

        // Same text should produce same embedding
        assert_eq!(embedding1, embedding2);

        // Different text should produce different embedding
        assert_ne!(embedding1, embedding3);

        // Check normalization
        let norm: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001); // Should be normalized to 1.0
    }
}
