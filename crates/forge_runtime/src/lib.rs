//! # Forge Runtime
//!
//! Runtime coordination for sessions, tasks, scheduling, and execution.
//!
//! This crate manages the runtime environment for executing inference and
//! training tasks, including session management, task scheduling, and
//! execution pipelines.

pub mod config;
pub mod logging;
pub mod runtime;
pub mod scheduler;
pub mod sessions;
pub mod tasks;
pub mod training_jobs;

// Re-export commonly used types
pub use runtime::{ChatRequest, Runtime, RuntimeError};
pub use scheduler::{Scheduler, SchedulerError};
pub use sessions::{Message, MessageRole, Session, SessionError, SessionManager};
pub use tasks::{JobStatus, Task, TaskPriority, TaskState, TaskType};
pub use training_jobs::{TrainingJobError, TrainingJobManager};

// Legacy in-memory implementations (kept for backward compatibility)
mod legacy {
    use chrono::{DateTime, Utc};
    use parking_lot::RwLock;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Arc;
    use thiserror::Error;
    use uuid::Uuid;

    /// Errors that can occur during legacy runtime operations.
    #[derive(Debug, Error)]
    pub enum LegacyRuntimeError {
        #[error("Session not found: {0}")]
        SessionNotFound(Uuid),

        #[error("Task not found: {0}")]
        TaskNotFound(Uuid),

        #[error("Task execution failed: {0}")]
        ExecutionFailed(String),

        #[error("Invalid task state transition")]
        InvalidStateTransition,
    }

    pub type Result<T> = std::result::Result<T, LegacyRuntimeError>;

    /// A legacy in-memory chat session.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LegacySession {
        pub id: Uuid,
        pub name: String,
        pub model_id: Option<Uuid>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub messages: Vec<LegacyMessage>,
    }

    /// A legacy message in a chat session.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LegacyMessage {
        pub id: Uuid,
        pub role: LegacyMessageRole,
        pub content: String,
        pub created_at: DateTime<Utc>,
    }

    /// Role of a legacy message sender.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum LegacyMessageRole {
        User,
        Assistant,
        System,
    }

    /// Legacy session manager (in-memory only).
    pub struct LegacySessionManager {
        sessions: Arc<RwLock<HashMap<Uuid, LegacySession>>>,
    }

    impl LegacySessionManager {
        /// Create a new legacy session manager.
        pub fn new() -> Self {
            Self {
                sessions: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        /// Create a new session.
        pub fn create_session(&self, name: String, model_id: Option<Uuid>) -> LegacySession {
            let session = LegacySession {
                id: Uuid::new_v4(),
                name,
                model_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                messages: Vec::new(),
            };

            let mut sessions = self.sessions.write();
            sessions.insert(session.id, session.clone());

            session
        }

        /// Get a session by ID.
        pub fn get_session(&self, id: &Uuid) -> Result<LegacySession> {
            let sessions = self.sessions.read();
            sessions
                .get(id)
                .cloned()
                .ok_or(LegacyRuntimeError::SessionNotFound(*id))
        }

        /// List all sessions.
        pub fn list_sessions(&self) -> Vec<LegacySession> {
            let sessions = self.sessions.read();
            sessions.values().cloned().collect()
        }

        /// Add a message to a session.
        pub fn add_message(
            &self,
            session_id: &Uuid,
            role: LegacyMessageRole,
            content: String,
        ) -> Result<LegacyMessage> {
            let mut sessions = self.sessions.write();
            let session = sessions
                .get_mut(session_id)
                .ok_or(LegacyRuntimeError::SessionNotFound(*session_id))?;

            let message = LegacyMessage {
                id: Uuid::new_v4(),
                role,
                content,
                created_at: Utc::now(),
            };

            session.messages.push(message.clone());
            session.updated_at = Utc::now();

            Ok(message)
        }

        /// Delete a session.
        pub fn delete_session(&self, id: &Uuid) -> Result<()> {
            let mut sessions = self.sessions.write();
            sessions
                .remove(id)
                .ok_or(LegacyRuntimeError::SessionNotFound(*id))?;
            Ok(())
        }
    }

    impl Default for LegacySessionManager {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Re-export legacy types for backward compatibility
pub use legacy::{
    LegacyMessage, LegacyMessageRole, LegacyRuntimeError, LegacySession, LegacySessionManager,
};

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::EchoBackend;
    use forge_store::Store;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_runtime_integration() {
        let store = Arc::new(Store::new_in_memory().await.unwrap());
        let backend = Arc::new(EchoBackend::new());
        let engine = Arc::new(forge_engine::Engine::new(backend));
        let models = Arc::new(forge_models::ModelRegistry::with_engine(
            store.clone(),
            engine.clone(),
        ));

        let runtime = Runtime::new(store, models, engine);

        // Test session creation
        let session = runtime
            .sessions()
            .create_session("Test Session".to_string(), None)
            .await
            .unwrap();

        assert_eq!(session.name, "Test Session");
    }

    #[tokio::test]
    async fn test_legacy_session_manager() {
        let manager = LegacySessionManager::new();
        let session = manager.create_session("Test".to_string(), None);

        let message = manager
            .add_message(&session.id, LegacyMessageRole::User, "Hello".to_string())
            .unwrap();

        assert_eq!(message.role, LegacyMessageRole::User);
        assert_eq!(message.content, "Hello");

        let updated_session = manager.get_session(&session.id).unwrap();
        assert_eq!(updated_session.messages.len(), 1);
    }
}
