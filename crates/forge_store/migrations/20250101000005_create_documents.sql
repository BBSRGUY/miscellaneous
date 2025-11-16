-- Create documents table
-- Stores documents for RAG (Retrieval-Augmented Generation)
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY NOT NULL,
    source TEXT NOT NULL, -- Source identifier (file path, URL, etc.)
    path TEXT, -- Path to processed document blob
    content TEXT, -- Extracted text content
    meta_json TEXT, -- JSON metadata (title, author, etc.)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    indexed_at TEXT -- When it was last indexed for vector search
);

CREATE INDEX idx_documents_source ON documents(source);
CREATE INDEX idx_documents_created_at ON documents(created_at);
CREATE INDEX idx_documents_indexed_at ON documents(indexed_at);
