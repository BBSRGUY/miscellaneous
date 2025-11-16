//! # Forge Store
//!
//! Database, migrations, blob storage, and repositories.
//!
//! This crate handles persistent storage for Forge, including SQLite database
//! management, migrations, and blob storage for models and datasets.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use tracing::log::LevelFilter;

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
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// Database connection pool and storage manager.
pub struct Store {
    pool: SqlitePool,
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

        let store = Self { pool };
        store.run_migrations().await?;

        Ok(store)
    }

    /// Create a new store with a file-based database.
    pub async fn new_with_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db_url = format!("sqlite:{}", path.as_ref().display());
        let options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Warn, std::time::Duration::from_secs(1));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let store = Self { pool };
        store.run_migrations().await?;

        Ok(store)
    }

    /// Run database migrations.
    async fn run_migrations(&self) -> Result<()> {
        // Create models table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                architecture TEXT,
                parameters INTEGER,
                context_length INTEGER,
                file_path TEXT,
                file_size INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                tags TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                model_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create messages table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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
        assert!(store.pool().is_closed() == false);
        store.close().await;
    }

    #[tokio::test]
    async fn test_create_file_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let store = Store::new_with_file(&db_path).await.unwrap();
        assert!(store.pool().is_closed() == false);
        assert!(db_path.exists());
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
        store.close().await;
    }
}
