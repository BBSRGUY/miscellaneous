//! # Task Types
//!
//! Task definitions for inference, training, and RAG operations.

use chrono::{DateTime, Utc};
use forge_engine::{InferenceRequest, TrainBatch, TrainConfig};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// State of a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    /// Task is queued and waiting
    Queued,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Priority of a task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Low priority (batch processing)
    Low = 0,
    /// Normal priority (standard operations)
    Normal = 1,
    /// High priority (user-facing operations)
    High = 2,
    /// Critical priority (time-sensitive operations)
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Type of task operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// Inference task (chat completion).
    Inference {
        /// Session ID for chat
        session_id: String,
        /// Model ID to use
        model_id: String,
        /// Inference request
        request: InferenceRequest,
    },
    /// Embedding generation task.
    Embedding {
        /// Text to embed
        text: String,
        /// Model ID to use
        model_id: String,
    },
    /// Training task.
    Training {
        /// Model ID to train
        model_id: String,
        /// Training batch
        batch: TrainBatch,
        /// Training configuration
        config: TrainConfig,
    },
    /// RAG document indexing task.
    RagIndexing {
        /// Document ID
        document_id: String,
        /// Document content
        content: String,
        /// Chunk size for splitting
        chunk_size: usize,
    },
}

/// A task to be executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: String,
    /// Type of task
    pub task_type: TaskType,
    /// Task priority
    pub priority: TaskPriority,
    /// Current state
    pub state: TaskState,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Start timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error: Option<String>,
}

impl Task {
    /// Create a new task.
    pub fn new(task_type: TaskType, priority: TaskPriority) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            priority,
            state: TaskState::Queued,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }

    /// Mark task as started.
    pub fn start(&mut self) {
        self.state = TaskState::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark task as completed.
    pub fn complete(&mut self) {
        self.state = TaskState::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark task as failed with error.
    pub fn fail(&mut self, error: String) {
        self.state = TaskState::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// Mark task as cancelled.
    pub fn cancel(&mut self) {
        self.state = TaskState::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Get task duration if completed.
    pub fn duration(&self) -> Option<std::time::Duration> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            completed.signed_duration_since(started)
                .to_std()
                .ok()
        } else {
            None
        }
    }
}

/// Job status returned to API clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// Job ID
    pub id: String,
    /// Current state
    pub state: TaskState,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Start time (if started)
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration (if completed)
    pub duration_ms: Option<u64>,
}

impl From<Task> for JobStatus {
    fn from(task: Task) -> Self {
        let duration_ms = task.duration()
            .map(|d| d.as_millis() as u64);

        Self {
            id: task.id,
            state: task.state,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            error: task.error,
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::SamplingParams;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        );

        assert_eq!(task.state, TaskState::Queued);
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::new(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        );

        // Start
        task.start();
        assert_eq!(task.state, TaskState::Running);
        assert!(task.started_at.is_some());

        // Complete
        task.complete();
        assert_eq!(task.state, TaskState::Completed);
        assert!(task.completed_at.is_some());
        assert!(task.duration().is_some());
    }

    #[test]
    fn test_task_failure() {
        let mut task = Task::new(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        );

        task.start();
        task.fail("Something went wrong".to_string());

        assert_eq!(task.state, TaskState::Failed);
        assert_eq!(task.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_job_status_conversion() {
        let mut task = Task::new(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        );

        task.start();
        task.complete();

        let status = JobStatus::from(task);
        assert_eq!(status.state, TaskState::Completed);
        assert!(status.duration_ms.is_some());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }
}
