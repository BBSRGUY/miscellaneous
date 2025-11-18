-- Add document_chunks table for RAG support
CREATE TABLE IF NOT EXISTS document_chunks (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB,
    metadata TEXT,
    token_count INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

-- Index for efficient chunk retrieval by document
CREATE INDEX IF NOT EXISTS idx_chunks_document ON document_chunks(document_id);

-- Index for chunk ordering
CREATE INDEX IF NOT EXISTS idx_chunks_order ON document_chunks(document_id, chunk_index);
