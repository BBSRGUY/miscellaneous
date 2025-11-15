//! Error types for the job engine

use thiserror::Error;

/// Result type for job operations
pub type JobResult<T> = Result<T, JobError>;

/// Errors that can occur during job execution
#[derive(Debug, Error)]
pub enum JobError {
    /// A step failed during execution
    #[error("Step '{step}' in job '{job}' failed: {message}")]
    StepFailed {
        job: String,
        step: String,
        message: String,
    },

    /// A job failed during execution
    #[error("Job '{job}' failed: {message}")]
    JobFailed { job: String, message: String },

    /// Circular dependency detected
    #[error("Circular dependency detected in job graph: {0}")]
    CircularDependency(String),

    /// Missing dependency
    #[error("Job '{job}' depends on '{dependency}' which does not exist")]
    MissingDependency { job: String, dependency: String },

    /// Invalid schedule expression
    #[error("Invalid schedule expression: {0}")]
    InvalidSchedule(String),

    /// Job not found
    #[error("Job '{0}' not found")]
    JobNotFound(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl JobError {
    /// Create a step failed error
    pub fn step_failed(job: impl Into<String>, step: impl Into<String>, message: impl Into<String>) -> Self {
        Self::StepFailed {
            job: job.into(),
            step: step.into(),
            message: message.into(),
        }
    }

    /// Create a job failed error
    pub fn job_failed(job: impl Into<String>, message: impl Into<String>) -> Self {
        Self::JobFailed {
            job: job.into(),
            message: message.into(),
        }
    }

    /// Create a circular dependency error
    pub fn circular_dependency(message: impl Into<String>) -> Self {
        Self::CircularDependency(message.into())
    }

    /// Create a missing dependency error
    pub fn missing_dependency(job: impl Into<String>, dependency: impl Into<String>) -> Self {
        Self::MissingDependency {
            job: job.into(),
            dependency: dependency.into(),
        }
    }
}

/// Result type for step execution
pub type StepResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
