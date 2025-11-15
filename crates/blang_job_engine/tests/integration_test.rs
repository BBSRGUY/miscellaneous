//! Integration tests for the Blang job engine

use blang_job_engine::{
    BrowserJobScheduler, Job, JobEngine, JobError, Schedule, ScheduleInterval, Step, StepFn,
};
use std::sync::{Arc, Mutex};

#[test]
fn test_basic_job_execution_order() {
    // Test that jobs execute in the correct order based on dependencies
    let mut engine = JobEngine::new();

    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Job A
    let log_a = execution_log.clone();
    let mut job_a = Job::new("job_a");
    job_a.add_step(Step::new(
        "step_a",
        StepFn::Sync(Box::new(move || {
            log_a.lock().unwrap().push("job_a");
            Ok(())
        })),
    ));

    // Job B depends on A
    let log_b = execution_log.clone();
    let mut job_b = Job::new("job_b");
    job_b.depends_on("job_a");
    job_b.add_step(Step::new(
        "step_b",
        StepFn::Sync(Box::new(move || {
            log_b.lock().unwrap().push("job_b");
            Ok(())
        })),
    ));

    // Job C depends on B
    let log_c = execution_log.clone();
    let mut job_c = Job::new("job_c");
    job_c.depends_on("job_b");
    job_c.add_step(Step::new(
        "step_c",
        StepFn::Sync(Box::new(move || {
            log_c.lock().unwrap().push("job_c");
            Ok(())
        })),
    ));

    engine.add_job(job_a);
    engine.add_job(job_b);
    engine.add_job(job_c);

    assert!(engine.run_all().is_ok());

    let log = execution_log.lock().unwrap();
    assert_eq!(*log, vec!["job_a", "job_b", "job_c"]);
}

#[test]
fn test_complex_dependency_graph() {
    // Test a diamond-shaped dependency graph
    //     A
    //    / \
    //   B   C
    //    \ /
    //     D

    let mut engine = JobEngine::new();
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Job A (no dependencies)
    let log_a = execution_log.clone();
    let mut job_a = Job::new("job_a");
    job_a.add_step(Step::new(
        "step_a",
        StepFn::Sync(Box::new(move || {
            log_a.lock().unwrap().push("job_a");
            Ok(())
        })),
    ));

    // Job B depends on A
    let log_b = execution_log.clone();
    let mut job_b = Job::new("job_b");
    job_b.depends_on("job_a");
    job_b.add_step(Step::new(
        "step_b",
        StepFn::Sync(Box::new(move || {
            log_b.lock().unwrap().push("job_b");
            Ok(())
        })),
    ));

    // Job C depends on A
    let log_c = execution_log.clone();
    let mut job_c = Job::new("job_c");
    job_c.depends_on("job_a");
    job_c.add_step(Step::new(
        "step_c",
        StepFn::Sync(Box::new(move || {
            log_c.lock().unwrap().push("job_c");
            Ok(())
        })),
    ));

    // Job D depends on B and C
    let log_d = execution_log.clone();
    let mut job_d = Job::new("job_d");
    job_d.depends_on("job_b");
    job_d.depends_on("job_c");
    job_d.add_step(Step::new(
        "step_d",
        StepFn::Sync(Box::new(move || {
            log_d.lock().unwrap().push("job_d");
            Ok(())
        })),
    ));

    engine.add_job(job_a);
    engine.add_job(job_b);
    engine.add_job(job_c);
    engine.add_job(job_d);

    assert!(engine.run_all().is_ok());

    let log = execution_log.lock().unwrap();
    // A should run first, then B and C (in any order), then D
    assert_eq!(log[0], "job_a");
    assert!(log.contains(&"job_b"));
    assert!(log.contains(&"job_c"));
    assert_eq!(log[3], "job_d");
}

#[test]
fn test_step_failure_handling() {
    // Test that when a step fails, the job fails and subsequent jobs don't run
    let mut engine = JobEngine::new();
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Job A that will fail
    let log_a = execution_log.clone();
    let mut job_a = Job::new("job_a");
    job_a.add_step(Step::new(
        "failing_step",
        StepFn::Sync(Box::new(move || {
            log_a.lock().unwrap().push("job_a");
            Err("Intentional failure".into())
        })),
    ));

    // Job B depends on A (should not run)
    let log_b = execution_log.clone();
    let mut job_b = Job::new("job_b");
    job_b.depends_on("job_a");
    job_b.add_step(Step::new(
        "step_b",
        StepFn::Sync(Box::new(move || {
            log_b.lock().unwrap().push("job_b");
            Ok(())
        })),
    ));

    engine.add_job(job_a);
    engine.add_job(job_b);

    let result = engine.run_all();
    assert!(result.is_err());

    if let Err(JobError::StepFailed { job, step, .. }) = result {
        assert_eq!(job, "job_a");
        assert_eq!(step, "failing_step");
    } else {
        panic!("Expected StepFailed error");
    }

    let log = execution_log.lock().unwrap();
    // Only job_a should have run
    assert_eq!(*log, vec!["job_a"]);
}

#[test]
fn test_continue_on_error() {
    // Test that steps can be configured to continue on error
    let mut engine = JobEngine::new();
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let mut job = Job::new("job");

    // Step 1: fails but continues
    let log_1 = execution_log.clone();
    let mut step_1 = Step::new(
        "step_1",
        StepFn::Sync(Box::new(move || {
            log_1.lock().unwrap().push("step_1");
            Err("Error but continue".into())
        })),
    );
    step_1.continue_on_error = true;
    job.add_step(step_1);

    // Step 2: should still run
    let log_2 = execution_log.clone();
    job.add_step(Step::new(
        "step_2",
        StepFn::Sync(Box::new(move || {
            log_2.lock().unwrap().push("step_2");
            Ok(())
        })),
    ));

    engine.add_job(job);

    // Should succeed overall
    assert!(engine.run_all().is_ok());

    let log = execution_log.lock().unwrap();
    assert_eq!(*log, vec!["step_1", "step_2"]);
}

#[test]
fn test_multiple_steps_in_job() {
    // Test that multiple steps execute sequentially in a single job
    let mut engine = JobEngine::new();
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    let mut job = Job::new("multi_step_job");

    for i in 1..=5 {
        let log = execution_log.clone();
        let step_name = format!("step_{}", i);
        job.add_step(Step::new(
            step_name.clone(),
            StepFn::Sync(Box::new(move || {
                log.lock().unwrap().push(format!("step_{}", i));
                Ok(())
            })),
        ));
    }

    engine.add_job(job);
    assert!(engine.run_all().is_ok());

    let log = execution_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["step_1", "step_2", "step_3", "step_4", "step_5"]
    );
}

#[test]
fn test_run_specific_job() {
    // Test running a specific job by name
    let mut engine = JobEngine::new();
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Job A
    let log_a = execution_log.clone();
    let mut job_a = Job::new("job_a");
    job_a.add_step(Step::new(
        "step_a",
        StepFn::Sync(Box::new(move || {
            log_a.lock().unwrap().push("job_a");
            Ok(())
        })),
    ));

    // Job B (independent)
    let log_b = execution_log.clone();
    let mut job_b = Job::new("job_b");
    job_b.add_step(Step::new(
        "step_b",
        StepFn::Sync(Box::new(move || {
            log_b.lock().unwrap().push("job_b");
            Ok(())
        })),
    ));

    engine.add_job(job_a);
    engine.add_job(job_b);

    // Run only job_a
    assert!(engine.run_job("job_a").is_ok());

    let log = execution_log.lock().unwrap();
    assert_eq!(*log, vec!["job_a"]);
}

#[test]
fn test_run_specific_job_with_dependencies() {
    // Test that running a specific job also runs its dependencies
    let mut engine = JobEngine::new();
    let execution_log = Arc::new(Mutex::new(Vec::new()));

    // Job A
    let log_a = execution_log.clone();
    let mut job_a = Job::new("job_a");
    job_a.add_step(Step::new(
        "step_a",
        StepFn::Sync(Box::new(move || {
            log_a.lock().unwrap().push("job_a");
            Ok(())
        })),
    ));

    // Job B depends on A
    let log_b = execution_log.clone();
    let mut job_b = Job::new("job_b");
    job_b.depends_on("job_a");
    job_b.add_step(Step::new(
        "step_b",
        StepFn::Sync(Box::new(move || {
            log_b.lock().unwrap().push("job_b");
            Ok(())
        })),
    ));

    // Job C (independent)
    let log_c = execution_log.clone();
    let mut job_c = Job::new("job_c");
    job_c.add_step(Step::new(
        "step_c",
        StepFn::Sync(Box::new(move || {
            log_c.lock().unwrap().push("job_c");
            Ok(())
        })),
    ));

    engine.add_job(job_a);
    engine.add_job(job_b);
    engine.add_job(job_c);

    // Run only job_b (should also run job_a)
    assert!(engine.run_job("job_b").is_ok());

    let log = execution_log.lock().unwrap();
    assert_eq!(*log, vec!["job_a", "job_b"]);
}

#[test]
fn test_execution_context() {
    // Test that execution context state is accessible across steps
    let mut engine = JobEngine::new();

    // Set some initial state
    engine.context_mut().set_state("key1", "value1");
    engine.context_mut().set_env("ENV_VAR", "test");

    // Verify state
    assert_eq!(
        engine.context().get_state("key1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        engine.context().get_env("ENV_VAR"),
        Some(&"test".to_string())
    );

    // Add a job that uses the context
    let mut job = Job::new("context_job");
    job.add_step(Step::new(
        "step1",
        StepFn::Sync(Box::new(|| Ok(()))),
    ));

    engine.add_job(job);
    assert!(engine.run_all().is_ok());
}

#[test]
fn test_schedule_parsing() {
    // Test schedule interval parsing using the parse_schedule function
    use blang_job_engine::schedule::parse_schedule;

    assert!(parse_schedule("30s").is_ok());
    assert!(parse_schedule("5m").is_ok());
    assert!(parse_schedule("2h").is_ok());
    assert!(parse_schedule("1d").is_ok());
    assert!(parse_schedule("once").is_ok());

    assert!(parse_schedule("invalid").is_err());
    assert!(parse_schedule("").is_err());
}

#[test]
fn test_schedule_to_duration() {
    // Test schedule interval to duration conversion
    let schedule_30s = Schedule::new(ScheduleInterval::Seconds(30));
    assert_eq!(schedule_30s.interval.to_duration().unwrap().as_secs(), 30);

    let schedule_5m = Schedule::new(ScheduleInterval::Minutes(5));
    assert_eq!(
        schedule_5m.interval.to_duration().unwrap().as_secs(),
        5 * 60
    );

    let schedule_2h = Schedule::new(ScheduleInterval::Hours(2));
    assert_eq!(
        schedule_2h.interval.to_duration().unwrap().as_secs(),
        2 * 60 * 60
    );

    let schedule_1d = Schedule::new(ScheduleInterval::Days(1));
    assert_eq!(
        schedule_1d.interval.to_duration().unwrap().as_secs(),
        24 * 60 * 60
    );
}

#[test]
fn test_browser_scheduler_creation() {
    // Test browser scheduler integration
    let engine = JobEngine::new();
    let scheduler = BrowserJobScheduler::new(engine);

    assert_eq!(scheduler.engine().job_count(), 0);
}

#[test]
fn test_browser_scheduler_with_schedule() {
    // Test browser scheduler with a scheduled engine
    let mut engine = JobEngine::new();

    let mut job = Job::new("scheduled_job");
    job.add_step(Step::new(
        "step1",
        StepFn::Sync(Box::new(|| Ok(()))),
    ));
    engine.add_job(job);

    let schedule = Schedule::new(ScheduleInterval::Seconds(30));
    engine.set_schedule(schedule);

    let scheduler = BrowserJobScheduler::new(engine);
    assert_eq!(scheduler.engine().job_count(), 1);
    assert!(scheduler.engine().schedule().is_some());
}

#[test]
fn test_circular_dependency_detection() {
    // Test that circular dependencies are detected
    let mut job_a = Job::new("job_a");
    job_a.depends_on("job_b");

    let mut job_b = Job::new("job_b");
    job_b.depends_on("job_a");

    let mut engine = JobEngine::new();
    engine.add_job(job_a);
    engine.add_job(job_b);

    let result = engine.run_all();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JobError::CircularDependency(_)));
}

#[test]
fn test_missing_dependency_detection() {
    // Test that missing dependencies are detected
    let mut job = Job::new("job_a");
    job.depends_on("nonexistent_job");

    let mut engine = JobEngine::new();
    engine.add_job(job);

    let result = engine.run_all();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), JobError::MissingDependency { .. }));
}

#[test]
fn test_job_reset() {
    // Test that jobs can be reset and re-executed
    let mut engine = JobEngine::new();
    let execution_count = Arc::new(Mutex::new(0));

    let count = execution_count.clone();
    let mut job = Job::new("job");
    job.add_step(Step::new(
        "step",
        StepFn::Sync(Box::new(move || {
            *count.lock().unwrap() += 1;
            Ok(())
        })),
    ));

    engine.add_job(job);

    // First execution
    assert!(engine.run_all().is_ok());
    assert_eq!(*execution_count.lock().unwrap(), 1);

    // Reset and execute again
    engine.reset();
    assert!(engine.run_all().is_ok());
    assert_eq!(*execution_count.lock().unwrap(), 2);
}
