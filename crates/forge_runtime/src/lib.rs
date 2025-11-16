//! # Forge Runtime
//!
//! Sessions, pipelines, scheduler, and task execution.
//!
//! This crate manages the runtime environment for executing inference and
//! training tasks, including session management, task scheduling, and
//! execution pipelines.

pub mod config;
pub mod logging;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during runtime operations.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid task state transition")]
    InvalidStateTransition,
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

/// A chat session with message history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub model_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

/// A message in a chat session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Role of a message sender.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Session manager for handling chat sessions.
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session.
    pub fn create_session(&self, name: String, model_id: Option<Uuid>) -> Session {
        let session = Session {
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
    pub fn get_session(&self, id: &Uuid) -> Result<Session> {
        let sessions = self.sessions.read();
        sessions
            .get(id)
            .cloned()
            .ok_or(RuntimeError::SessionNotFound(*id))
    }

    /// List all sessions.
    pub fn list_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read();
        sessions.values().cloned().collect()
    }

    /// Add a message to a session.
    pub fn add_message(&self, session_id: &Uuid, role: MessageRole, content: String) -> Result<Message> {
        let mut sessions = self.sessions.write();
        let session = sessions
            .get_mut(session_id)
            .ok_or(RuntimeError::SessionNotFound(*session_id))?;

        let message = Message {
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
            .ok_or(RuntimeError::SessionNotFound(*id))?;
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// State of a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Priority of a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A task to be executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Type of task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Inference { session_id: Uuid, prompt: String },
    Training { model_id: Uuid, dataset_path: String },
    Embedding { text: String },
}

/// Task scheduler for managing and prioritizing tasks.
pub struct TaskScheduler {
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
    queue: Arc<RwLock<VecDeque<Uuid>>>,
}

impl TaskScheduler {
    /// Create a new task scheduler.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Submit a new task.
    pub fn submit(&self, task_type: TaskType, priority: TaskPriority) -> Uuid {
        let task = Task {
            id: Uuid::new_v4(),
            task_type,
            priority,
            state: TaskState::Queued,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        let id = task.id;

        let mut tasks = self.tasks.write();
        tasks.insert(id, task);

        let mut queue = self.queue.write();
        // Simple FIFO for now; could implement priority-based insertion
        queue.push_back(id);

        id
    }

    /// Get the next task from the queue.
    pub fn next_task(&self) -> Option<Task> {
        let mut queue = self.queue.write();
        let id = queue.pop_front()?;

        let mut tasks = self.tasks.write();
        let task = tasks.get_mut(&id)?;
        task.state = TaskState::Running;
        task.started_at = Some(Utc::now());

        Some(task.clone())
    }

    /// Mark a task as completed.
    pub fn complete_task(&self, id: &Uuid) -> Result<()> {
        let mut tasks = self.tasks.write();
        let task = tasks
            .get_mut(id)
            .ok_or(RuntimeError::TaskNotFound(*id))?;

        task.state = TaskState::Completed;
        task.completed_at = Some(Utc::now());

        Ok(())
    }

    /// Mark a task as failed.
    pub fn fail_task(&self, id: &Uuid, _error: String) -> Result<()> {
        let mut tasks = self.tasks.write();
        let task = tasks
            .get_mut(id)
            .ok_or(RuntimeError::TaskNotFound(*id))?;

        task.state = TaskState::Failed;
        task.completed_at = Some(Utc::now());

        Ok(())
    }

    /// Get task status.
    pub fn get_task(&self, id: &Uuid) -> Result<Task> {
        let tasks = self.tasks.read();
        tasks
            .get(id)
            .cloned()
            .ok_or(RuntimeError::TaskNotFound(*id))
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new();
        let session = manager.create_session("Test Session".to_string(), None);

        assert_eq!(session.name, "Test Session");
        assert_eq!(session.messages.len(), 0);
    }

    #[test]
    fn test_add_message() {
        let manager = SessionManager::new();
        let session = manager.create_session("Test".to_string(), None);

        let message = manager
            .add_message(&session.id, MessageRole::User, "Hello".to_string())
            .unwrap();

        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello");

        let updated_session = manager.get_session(&session.id).unwrap();
        assert_eq!(updated_session.messages.len(), 1);
    }

    #[test]
    fn test_task_scheduling() {
        let scheduler = TaskScheduler::new();
        let task_id = scheduler.submit(
            TaskType::Inference {
                session_id: Uuid::new_v4(),
                prompt: "Test".to_string(),
            },
            TaskPriority::Normal,
        );

        let task = scheduler.next_task().unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.state, TaskState::Running);

        scheduler.complete_task(&task_id).unwrap();
        let completed = scheduler.get_task(&task_id).unwrap();
        assert_eq!(completed.state, TaskState::Completed);
    }
}
