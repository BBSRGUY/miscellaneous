-- Create models table
-- Stores metadata about LLM models
CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    path TEXT,
    backend TEXT NOT NULL,
    format TEXT NOT NULL,
    size_bytes INTEGER,
    quantization TEXT,
    tags TEXT, -- JSON array of tags
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_models_name ON models(name);
CREATE INDEX idx_models_backend ON models(backend);
CREATE INDEX idx_models_created_at ON models(created_at);
