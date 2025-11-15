//! Blang Job Execution Engine
//!
//! This crate provides a runtime engine for executing Blang scripts with jobs and steps.
//! It supports:
//! - Sequential step execution within jobs
//! - Job dependency resolution (topological sort)
//! - Parallel step execution
//! - Error handling and recovery
//! - Basic scheduling support
//!
//! # Architecture
//!
//! ```text
//! Script
//!   ├── Config (key-value options)
//!   ├── Schedule (cron expressions, intervals)
//!   ├── State (shared state across jobs)
//!   └── Jobs
//!       ├── Job A
//!       │   ├── Step 1 (sequential)
//!       │   ├── Step 2 (sequential)
//!       │   └── Parallel { Step 3a, Step 3b }
//!       └── Job B (after Job A)
//!           └── Step 1
//! ```
//!
//! # Example
//!
//! ```rust
//! use blang_job_engine::{JobEngine, Job, Step, StepFn};
//!
//! let mut engine = JobEngine::new();
//!
//! // Define a job with steps
//! let mut job = Job::new("deploy");
//! job.add_step(Step::new("build", StepFn::Sync(Box::new(|| {
//!     println!("Building...");
//!     Ok(())
//! }))));
//! job.add_step(Step::new("test", StepFn::Sync(Box::new(|| {
//!     println!("Testing...");
//!     Ok(())
//! }))));
//!
//! engine.add_job(job);
//! engine.run_all().unwrap();
//! ```

pub mod browser;
pub mod error;
pub mod executor;
pub mod job;
pub mod schedule;
pub mod step;

pub use browser::BrowserJobScheduler;
pub use error::{JobError, JobResult, StepResult};
pub use executor::JobEngine;
pub use job::Job;
pub use schedule::{Schedule, ScheduleInterval};
pub use step::{Step, StepFn};

// Re-exports for convenience
pub use executor::ExecutionContext;
