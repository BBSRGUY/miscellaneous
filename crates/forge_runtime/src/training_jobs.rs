//! Training job management.

use forge_engine::training::{TrainConfig, Trainer, TrainingError, TrainingJob, TrainingState};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Errors that can occur during training job management.
#[derive(Debug, Error)]
pub enum TrainingJobError {
    #[error("Training job not found: {0}")]
    JobNotFound(String),

    #[error("Training error: {0}")]
    TrainingError(#[from] TrainingError),

    #[error("Job already exists: {0}")]
    JobAlreadyExists(String),

    #[error("Invalid state transition")]
    InvalidStateTransition,
}

pub type Result<T> = std::result::Result<T, TrainingJobError>;

/// Manager for training jobs.
pub struct TrainingJobManager {
    jobs: Arc<RwLock<HashMap<String, Arc<RwLock<TrainingJob>>>>>,
    trainers: Arc<RwLock<HashMap<String, Arc<Trainer>>>>,
}

impl TrainingJobManager {
    /// Create a new training job manager.
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            trainers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new training job.
    pub fn create_job(&self, config: TrainConfig) -> Result<String> {
        // Validate configuration
        config.validate()?;

        // Generate job ID
        let job_id = format!("train-{}", Uuid::new_v4());

        // Check if job already exists
        {
            let jobs = self.jobs.read();
            if jobs.contains_key(&job_id) {
                return Err(TrainingJobError::JobAlreadyExists(job_id));
            }
        }

        // Estimate dataset size for total steps calculation
        // In a real implementation, this would properly load the dataset
        let dataset_size = 1000; // Default estimate
        let total_steps = config.total_steps(dataset_size);

        // Create training job
        let job = TrainingJob::new(job_id.clone(), config, total_steps);

        info!("Created training job: {}", job_id);

        // Store job
        {
            let mut jobs = self.jobs.write();
            jobs.insert(job_id.clone(), Arc::new(RwLock::new(job)));
        }

        Ok(job_id)
    }

    /// Get a training job by ID.
    pub fn get_job(&self, job_id: &str) -> Result<TrainingJob> {
        let jobs = self.jobs.read();
        let job_ref = jobs
            .get(job_id)
            .ok_or_else(|| TrainingJobError::JobNotFound(job_id.to_string()))?;

        Ok(job_ref.read().clone())
    }

    /// List all training jobs.
    pub fn list_jobs(&self) -> Vec<TrainingJob> {
        let jobs = self.jobs.read();
        jobs.values().map(|j| j.read().clone()).collect()
    }

    /// Start a training job.
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let job_ref = {
            let jobs = self.jobs.read();
            jobs.get(job_id)
                .ok_or_else(|| TrainingJobError::JobNotFound(job_id.to_string()))?
                .clone()
        };

        // Get job config
        let config = {
            let job = job_ref.read();
            job.config.clone()
        };

        // Create trainer
        let job_for_trainer = job_ref.read().clone();
        let trainer = Arc::new(Trainer::new(job_for_trainer));

        // Store trainer reference
        {
            let mut trainers = self.trainers.write();
            trainers.insert(job_id.to_string(), trainer.clone());
        }

        // Spawn training task
        let job_id_owned = job_id.to_string();
        let trainer_clone = trainer.clone();
        let job_ref_clone = job_ref.clone();

        tokio::spawn(async move {
            info!("Starting training job: {}", job_id_owned);

            match trainer_clone.train().await {
                Ok(()) => {
                    info!("Training job completed successfully: {}", job_id_owned);
                }
                Err(e) => {
                    warn!("Training job failed: {}: {}", job_id_owned, e);
                    let mut job = job_ref_clone.write();
                    job.mark_failed(e.to_string());
                }
            }
        });

        Ok(())
    }

    /// Cancel a training job.
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        // Get trainer
        let trainer = {
            let trainers = self.trainers.read();
            trainers
                .get(job_id)
                .ok_or_else(|| TrainingJobError::JobNotFound(job_id.to_string()))?
                .clone()
        };

        // Cancel training
        trainer.cancel().await?;

        info!("Cancelled training job: {}", job_id);

        Ok(())
    }

    /// Delete a training job.
    pub fn delete_job(&self, job_id: &str) -> Result<()> {
        let job = {
            let jobs = self.jobs.read();
            jobs.get(job_id)
                .ok_or_else(|| TrainingJobError::JobNotFound(job_id.to_string()))?
                .read()
                .clone()
        };

        // Only allow deletion of completed/failed/cancelled jobs
        if matches!(job.state, TrainingState::Running | TrainingState::Initializing) {
            return Err(TrainingJobError::InvalidStateTransition);
        }

        // Remove job and trainer
        {
            let mut jobs = self.jobs.write();
            jobs.remove(job_id);
        }

        {
            let mut trainers = self.trainers.write();
            trainers.remove(job_id);
        }

        info!("Deleted training job: {}", job_id);

        Ok(())
    }

    /// Get logs for a training job.
    pub fn get_logs(&self, job_id: &str) -> Result<Vec<String>> {
        let job = self.get_job(job_id)?;

        // Convert metrics to log lines
        let logs: Vec<String> = job
            .metrics_history
            .iter()
            .map(|m| {
                format!(
                    "[Step {}] epoch={:.2}, loss={:.4}, lr={:.6}",
                    m.step, m.epoch, m.loss, m.learning_rate
                )
            })
            .collect();

        Ok(logs)
    }
}

impl Default for TrainingJobManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_create_and_get_job() {
        let manager = TrainingJobManager::new();

        // Create a minimal config
        let mut config = TrainConfig::default();
        config.base_model_id = "test-model".to_string();
        config.dataset_path = PathBuf::from("/tmp/test.jsonl");

        // Note: This will fail validation because dataset doesn't exist
        // In a real test, we'd create a temporary dataset file
        let result = manager.create_job(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_jobs() {
        let manager = TrainingJobManager::new();
        let jobs = manager.list_jobs();
        assert_eq!(jobs.len(), 0);
    }
}
