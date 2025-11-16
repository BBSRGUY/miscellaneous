-- Create jobs table
-- Stores background jobs (training, inference, etc.)
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL, -- 'inference', 'training', 'embedding', etc.
    status TEXT NOT NULL, -- 'queued', 'running', 'completed', 'failed', 'cancelled'
    progress REAL DEFAULT 0.0, -- 0.0 to 1.0
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    config_json TEXT, -- JSON configuration for the job
    logs_path TEXT, -- Path to log file
    error_message TEXT
);

CREATE INDEX idx_jobs_kind ON jobs(kind);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at);
