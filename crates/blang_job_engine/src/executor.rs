//! Job execution engine

use crate::error::{JobError, JobResult};
use crate::job::{topological_sort, Job};
use crate::schedule::Schedule;
use std::collections::HashMap;

/// Execution context shared across jobs
pub struct ExecutionContext {
    /// Global state
    pub state: HashMap<String, String>,

    /// Environment variables
    pub env: HashMap<String, String>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            env: HashMap::new(),
        }
    }

    /// Set a state value
    pub fn set_state(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.state.insert(key.into(), value.into());
    }

    /// Get a state value
    pub fn get_state(&self, key: &str) -> Option<&String> {
        self.state.get(key)
    }

    /// Set an environment variable
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }

    /// Get an environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// The main job execution engine
pub struct JobEngine {
    /// All registered jobs
    jobs: Vec<Job>,

    /// Execution context
    context: ExecutionContext,

    /// Schedule (if any)
    schedule: Option<Schedule>,

    /// Configuration options
    config: HashMap<String, String>,
}

impl JobEngine {
    /// Create a new job engine
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            context: ExecutionContext::new(),
            schedule: None,
            config: HashMap::new(),
        }
    }

    /// Add a job to the engine
    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    /// Set a configuration option
    pub fn set_config(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.config.insert(key.into(), value.into());
    }

    /// Get a configuration option
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Set the schedule
    pub fn set_schedule(&mut self, schedule: Schedule) {
        self.schedule = Some(schedule);
    }

    /// Get the schedule
    pub fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    /// Get the execution context
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Get a mutable reference to the execution context
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }

    /// Run all jobs in dependency order
    pub fn run_all(&mut self) -> JobResult<()> {
        // Get execution order based on dependencies
        let order = topological_sort(&self.jobs)?;

        // Execute jobs in order
        for &idx in &order {
            let job = &mut self.jobs[idx];
            job.execute()?;
        }

        Ok(())
    }

    /// Run a specific job by name
    pub fn run_job(&mut self, name: &str) -> JobResult<()> {
        // Find the job
        let job_idx = self
            .jobs
            .iter()
            .position(|j| j.name == name)
            .ok_or_else(|| JobError::JobNotFound(name.to_string()))?;

        // Check dependencies
        let dependencies: Vec<String> = self.jobs[job_idx].dependencies().to_vec();

        // Execute dependencies first
        for dep in dependencies {
            self.run_job(&dep)?;
        }

        // Execute the job
        self.jobs[job_idx].execute()
    }

    /// Reset all jobs (mark as not executed)
    pub fn reset(&mut self) {
        for job in &mut self.jobs {
            job.reset();
        }
    }

    /// Get the number of jobs
    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }

    /// Check if a job exists
    pub fn has_job(&self, name: &str) -> bool {
        self.jobs.iter().any(|j| j.name == name)
    }

    /// Get a job by name
    pub fn get_job(&self, name: &str) -> Option<&Job> {
        self.jobs.iter().find(|j| j.name == name)
    }

    /// Get a mutable job by name
    pub fn get_job_mut(&mut self, name: &str) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.name == name)
    }
}

impl Default for JobEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::{Step, StepFn};

    #[test]
    fn test_job_engine_execution() {
        let mut engine = JobEngine::new();

        let mut job1 = Job::new("job1");
        job1.add_step(Step::new("step1", StepFn::Sync(Box::new(|| Ok(())))));

        let mut job2 = Job::new("job2");
        job2.depends_on("job1");
        job2.add_step(Step::new("step2", StepFn::Sync(Box::new(|| Ok(())))));

        engine.add_job(job1);
        engine.add_job(job2);

        assert!(engine.run_all().is_ok());
        assert_eq!(engine.job_count(), 2);
    }

    #[test]
    fn test_run_specific_job() {
        let mut engine = JobEngine::new();

        let mut job1 = Job::new("job1");
        job1.add_step(Step::new("step1", StepFn::Sync(Box::new(|| Ok(())))));

        let mut job2 = Job::new("job2");
        job2.add_step(Step::new("step2", StepFn::Sync(Box::new(|| Ok(())))));

        engine.add_job(job1);
        engine.add_job(job2);

        assert!(engine.run_job("job1").is_ok());
        assert!(engine.get_job("job1").unwrap().is_executed());
        assert!(!engine.get_job("job2").unwrap().is_executed());
    }

    #[test]
    fn test_context() {
        let mut engine = JobEngine::new();

        engine.context_mut().set_state("key", "value");
        assert_eq!(engine.context().get_state("key"), Some(&"value".to_string()));

        engine.context_mut().set_env("ENV_VAR", "test");
        assert_eq!(engine.context().get_env("ENV_VAR"), Some(&"test".to_string()));
    }
}
