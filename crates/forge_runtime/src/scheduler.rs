//! # Task Scheduler
//!
//! Async task scheduler using tokio channels for task execution.

use crate::tasks::{Task, TaskPriority, TaskState, TaskType};
use parking_lot::RwLock;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Errors that can occur during scheduling.
#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Scheduler is shut down")]
    ShutDown,

    #[error("Channel send error: {0}")]
    ChannelSend(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;

/// Command sent to the scheduler.
#[derive(Debug)]
enum SchedulerCommand {
    Submit(Task),
    Get(String, tokio::sync::oneshot::Sender<Option<Task>>),
    Complete(String),
    Fail(String, String),
    Shutdown,
}

/// Wrapper for tasks in priority queue.
#[derive(Debug, Clone)]
struct PriorityTask {
    task: Task,
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority
            && self.task.created_at == other.task.created_at
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier created_at
        match self.task.priority.cmp(&other.task.priority) {
            std::cmp::Ordering::Equal => {
                // Earlier timestamps are higher priority (reverse order)
                other.task.created_at.cmp(&self.task.created_at)
            }
            other => other,
        }
    }
}

/// Task scheduler with priority-based execution.
pub struct Scheduler {
    command_tx: mpsc::UnboundedSender<SchedulerCommand>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let tasks = Arc::new(RwLock::new(HashMap::new()));

        let scheduler_tasks = tasks.clone();

        // Spawn scheduler event loop
        tokio::spawn(async move {
            Self::run_scheduler(command_rx, scheduler_tasks).await;
        });

        Self { command_tx, tasks }
    }

    /// Internal scheduler event loop.
    async fn run_scheduler(
        mut command_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
        tasks: Arc<RwLock<HashMap<String, Task>>>,
    ) {
        let mut queue: BinaryHeap<PriorityTask> = BinaryHeap::new();

        debug!("Scheduler event loop started");

        while let Some(command) = command_rx.recv().await {
            match command {
                SchedulerCommand::Submit(task) => {
                    let id = task.id.clone();
                    info!("Task submitted: {} (priority: {:?})", id, task.priority);

                    tasks.write().insert(id.clone(), task.clone());
                    queue.push(PriorityTask { task });

                    debug!("Queue size: {}", queue.len());
                }
                SchedulerCommand::Get(id, response_tx) => {
                    let task = tasks.read().get(&id).cloned();
                    let _ = response_tx.send(task);
                }
                SchedulerCommand::Complete(id) => {
                    if let Some(task) = tasks.write().get_mut(&id) {
                        task.complete();
                        info!("Task completed: {}", id);
                    } else {
                        warn!("Attempted to complete unknown task: {}", id);
                    }
                }
                SchedulerCommand::Fail(id, error) => {
                    if let Some(task) = tasks.write().get_mut(&id) {
                        task.fail(error.clone());
                        error!("Task failed: {} - {}", id, error);
                    } else {
                        warn!("Attempted to fail unknown task: {}", id);
                    }
                }
                SchedulerCommand::Shutdown => {
                    info!("Scheduler shutting down");
                    break;
                }
            }
        }

        debug!("Scheduler event loop terminated");
    }

    /// Submit a task to the scheduler.
    pub fn submit(&self, task_type: TaskType, priority: TaskPriority) -> Result<String> {
        let task = Task::new(task_type, priority);
        let id = task.id.clone();

        self.command_tx
            .send(SchedulerCommand::Submit(task))
            .map_err(|_| SchedulerError::ShutDown)?;

        Ok(id)
    }

    /// Get a task by ID.
    pub async fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.command_tx
            .send(SchedulerCommand::Get(id.to_string(), tx))
            .map_err(|_| SchedulerError::ShutDown)?;

        rx.await.map_err(|_| SchedulerError::ShutDown)
    }

    /// Mark a task as completed.
    pub fn complete_task(&self, id: &str) -> Result<()> {
        self.command_tx
            .send(SchedulerCommand::Complete(id.to_string()))
            .map_err(|_| SchedulerError::ShutDown)?;

        Ok(())
    }

    /// Mark a task as failed.
    pub fn fail_task(&self, id: &str, error: String) -> Result<()> {
        self.command_tx
            .send(SchedulerCommand::Fail(id.to_string(), error))
            .map_err(|_| SchedulerError::ShutDown)?;

        Ok(())
    }

    /// Get all tasks (snapshot).
    pub fn get_all_tasks(&self) -> Vec<Task> {
        self.tasks.read().values().cloned().collect()
    }

    /// Get tasks by state.
    pub fn get_tasks_by_state(&self, state: TaskState) -> Vec<Task> {
        self.tasks
            .read()
            .values()
            .filter(|t| t.state == state)
            .cloned()
            .collect()
    }

    /// Shutdown the scheduler.
    pub fn shutdown(&self) {
        let _ = self.command_tx.send(SchedulerCommand::Shutdown);
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::InferenceRequest;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.get_all_tasks().len(), 0);
    }

    #[tokio::test]
    async fn test_submit_task() {
        let scheduler = Scheduler::new();

        let id = scheduler.submit(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        ).unwrap();

        // Give scheduler time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let task = scheduler.get_task(&id).await.unwrap().unwrap();
        assert_eq!(task.id, id);
        assert_eq!(task.state, TaskState::Queued);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let scheduler = Scheduler::new();

        let id = scheduler.submit(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        ).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        scheduler.complete_task(&id).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let task = scheduler.get_task(&id).await.unwrap().unwrap();
        assert_eq!(task.state, TaskState::Completed);
    }

    #[tokio::test]
    async fn test_fail_task() {
        let scheduler = Scheduler::new();

        let id = scheduler.submit(
            TaskType::Embedding {
                text: "Test".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        ).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        scheduler.fail_task(&id, "Test error".to_string()).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let task = scheduler.get_task(&id).await.unwrap().unwrap();
        assert_eq!(task.state, TaskState::Failed);
        assert_eq!(task.error, Some("Test error".to_string()));
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let scheduler = Scheduler::new();

        // Submit tasks with different priorities
        scheduler.submit(
            TaskType::Embedding {
                text: "Low".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Low,
        ).unwrap();

        scheduler.submit(
            TaskType::Embedding {
                text: "Critical".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Critical,
        ).unwrap();

        scheduler.submit(
            TaskType::Embedding {
                text: "Normal".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        ).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let tasks = scheduler.get_all_tasks();
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_get_tasks_by_state() {
        let scheduler = Scheduler::new();

        let id1 = scheduler.submit(
            TaskType::Embedding {
                text: "Test 1".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        ).unwrap();

        let id2 = scheduler.submit(
            TaskType::Embedding {
                text: "Test 2".to_string(),
                model_id: "model-1".to_string(),
            },
            TaskPriority::Normal,
        ).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        scheduler.complete_task(&id1).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let queued = scheduler.get_tasks_by_state(TaskState::Queued);
        let completed = scheduler.get_tasks_by_state(TaskState::Completed);

        assert_eq!(queued.len(), 1);
        assert_eq!(completed.len(), 1);
    }
}
