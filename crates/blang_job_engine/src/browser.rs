//! Browser-specific job scheduling integration
//!
//! This module provides integration between the job engine and browser timers
//! (setTimeout, setInterval) via the blang_runtime timer primitives.

use crate::error::{JobError, JobResult};
use crate::executor::JobEngine;
use crate::schedule::{Schedule, ScheduleInterval};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use blang_runtime::timer::{set_interval, set_timeout};

/// Timer ID type (u32 to match browser timer APIs)
pub type TimerId = u32;

/// Browser-based job scheduler
pub struct BrowserJobScheduler {
    /// The job engine to schedule
    engine: JobEngine,

    /// Active timer IDs for each scheduled job
    timers: HashMap<String, TimerId>,

    /// Callback indices registered for each job
    callbacks: HashMap<String, u32>,
}

impl BrowserJobScheduler {
    /// Create a new browser job scheduler
    pub fn new(engine: JobEngine) -> Self {
        Self {
            engine,
            timers: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    /// Schedule all jobs based on their schedules
    #[cfg(target_arch = "wasm32")]
    pub fn schedule_all(&mut self) -> JobResult<()> {
        // In a real implementation, this would:
        // 1. Check each job's schedule configuration
        // 2. Register callbacks for timer events
        // 3. Set up timers using set_timeout or set_interval
        // 4. Store timer IDs for later cancellation

        if let Some(schedule) = self.engine.schedule() {
            if !schedule.enabled {
                return Ok(());
            }

            match &schedule.interval {
                ScheduleInterval::Once => {
                    // Use setTimeout to run once
                    // In a real implementation:
                    // let timer_id = set_timeout(callback_index, 0);
                }
                ScheduleInterval::Seconds(s)
                | ScheduleInterval::Minutes(s)
                | ScheduleInterval::Hours(s)
                | ScheduleInterval::Days(s) => {
                    // Use setInterval for recurring schedules
                    // let duration_ms = schedule.interval.to_duration()?.as_millis() as u32;
                    // let timer_id = set_interval(callback_index, duration_ms);
                }
                #[cfg(feature = "scheduling")]
                ScheduleInterval::Cron(_) => {
                    return Err(JobError::Other(
                        "Cron scheduling requires external timer management".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Schedule all jobs (non-WASM fallback)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn schedule_all(&mut self) -> JobResult<()> {
        Err(JobError::Other(
            "Browser scheduling is only available on WASM target".into(),
        ))
    }

    /// Schedule a specific job by name
    #[cfg(target_arch = "wasm32")]
    pub fn schedule_job(&mut self, job_name: &str, schedule: Schedule) -> JobResult<()> {
        if !schedule.enabled {
            return Ok(());
        }

        // Generate a callback index (in a real impl, this would be managed globally)
        let callback_index = self.callbacks.len() as u32;
        self.callbacks.insert(job_name.to_string(), callback_index);

        // Calculate delay in milliseconds
        let delay_ms = schedule.interval.to_duration()?.as_millis() as u32;

        // Set up the timer
        let timer_id = match schedule.interval {
            ScheduleInterval::Once => {
                // For a real implementation:
                // set_timeout(callback_index, delay_ms)
                0 // Placeholder
            }
            ScheduleInterval::Seconds(_)
            | ScheduleInterval::Minutes(_)
            | ScheduleInterval::Hours(_)
            | ScheduleInterval::Days(_) => {
                // For a real implementation:
                // set_interval(callback_index, delay_ms)
                0 // Placeholder
            }
            #[cfg(feature = "scheduling")]
            ScheduleInterval::Cron(_) => {
                return Err(JobError::Other(
                    "Cron scheduling requires external timer management".into(),
                ));
            }
        };

        self.timers.insert(job_name.to_string(), timer_id);
        Ok(())
    }

    /// Schedule a specific job (non-WASM fallback)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn schedule_job(&mut self, _job_name: &str, _schedule: Schedule) -> JobResult<()> {
        Err(JobError::Other(
            "Browser scheduling is only available on WASM target".into(),
        ))
    }

    /// Cancel a scheduled job
    #[cfg(target_arch = "wasm32")]
    pub fn cancel_job(&mut self, job_name: &str) -> JobResult<()> {
        if let Some(timer_id) = self.timers.remove(job_name) {
            // In a real implementation:
            // clear_interval(timer_id) or clear_timeout(timer_id)
            self.callbacks.remove(job_name);
        }
        Ok(())
    }

    /// Cancel a scheduled job (non-WASM fallback)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn cancel_job(&mut self, _job_name: &str) -> JobResult<()> {
        Ok(())
    }

    /// Get the underlying job engine
    pub fn engine(&self) -> &JobEngine {
        &self.engine
    }

    /// Get a mutable reference to the underlying job engine
    pub fn engine_mut(&mut self) -> &mut JobEngine {
        &mut self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::Job;
    use crate::step::{Step, StepFn};

    #[test]
    fn test_browser_scheduler_creation() {
        let engine = JobEngine::new();
        let scheduler = BrowserJobScheduler::new(engine);

        assert_eq!(scheduler.engine().job_count(), 0);
    }

    #[test]
    fn test_browser_scheduler_with_jobs() {
        let mut engine = JobEngine::new();

        let mut job = Job::new("test_job");
        job.add_step(Step::new("step1", StepFn::Sync(Box::new(|| Ok(())))));
        engine.add_job(job);

        let scheduler = BrowserJobScheduler::new(engine);
        assert_eq!(scheduler.engine().job_count(), 1);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_schedule_all_non_wasm() {
        let engine = JobEngine::new();
        let mut scheduler = BrowserJobScheduler::new(engine);

        let result = scheduler.schedule_all();
        assert!(result.is_err());
    }
}
