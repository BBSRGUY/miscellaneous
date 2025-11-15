//! Job definition and execution

use crate::error::{JobError, JobResult};
use crate::step::{Step, StepContext};

/// A job containing multiple steps
pub struct Job {
    /// Job name
    pub name: String,

    /// Steps to execute
    pub steps: Vec<Step>,

    /// Jobs this depends on
    pub depends_on: Vec<String>,

    /// Whether this job has been executed
    executed: bool,
}

impl Job {
    /// Create a new job
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            depends_on: Vec::new(),
            executed: false,
        }
    }

    /// Add a step to this job
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Add a dependency
    pub fn depends_on(&mut self, job_name: impl Into<String>) {
        self.depends_on.push(job_name.into());
    }

    /// Check if this job has been executed
    pub fn is_executed(&self) -> bool {
        self.executed
    }

    /// Mark this job as executed
    pub fn mark_executed(&mut self) {
        self.executed = true;
    }

    /// Reset execution state
    pub fn reset(&mut self) {
        self.executed = false;
    }

    /// Execute all steps in this job
    pub fn execute(&mut self) -> JobResult<()> {
        if self.executed {
            return Ok(());
        }

        for step in &mut self.steps {
            let mut context = StepContext::new(self.name.clone(), step.name.clone());

            match step.execute(&mut context) {
                Ok(()) => {
                    // Step succeeded
                }
                Err(e) => {
                    if step.continue_on_error {
                        eprintln!("Step '{}' failed but continuing: {}", step.name, e);
                    } else {
                        return Err(JobError::step_failed(
                            &self.name,
                            &step.name,
                            e.to_string(),
                        ));
                    }
                }
            }
        }

        self.executed = true;
        Ok(())
    }

    /// Get dependencies of this job
    pub fn dependencies(&self) -> &[String] {
        &self.depends_on
    }
}

/// Topologically sort jobs based on dependencies
pub fn topological_sort(jobs: &[Job]) -> JobResult<Vec<usize>> {
    let n = jobs.len();
    let mut in_degree = vec![0; n];
    let mut adj_list: Vec<Vec<usize>> = vec![Vec::new(); n];

    // Build adjacency list and calculate in-degrees
    for (i, job) in jobs.iter().enumerate() {
        for dep in &job.depends_on {
            // Find the dependency job index
            if let Some(dep_idx) = jobs.iter().position(|j| &j.name == dep) {
                adj_list[dep_idx].push(i);
                in_degree[i] += 1;
            } else {
                return Err(JobError::missing_dependency(&job.name, dep));
            }
        }
    }

    // Kahn's algorithm for topological sort
    let mut queue: Vec<usize> = Vec::new();
    for (i, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            queue.push(i);
        }
    }

    let mut result = Vec::new();
    let mut visited = 0;

    while let Some(idx) = queue.pop() {
        result.push(idx);
        visited += 1;

        for &neighbor in &adj_list[idx] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push(neighbor);
            }
        }
    }

    if visited != n {
        return Err(JobError::circular_dependency(
            "Circular dependency detected in job graph",
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::StepFn;

    #[test]
    fn test_job_execution() {
        let mut job = Job::new("test_job");

        job.add_step(Step::new(
            "step1",
            StepFn::Sync(Box::new(|| {
                Ok(())
            })),
        ));

        assert!(job.execute().is_ok());
        assert!(job.is_executed());
    }

    #[test]
    fn test_topological_sort() {
        let job_a = Job::new("a");
        let mut job_b = Job::new("b");
        let mut job_c = Job::new("c");

        job_b.depends_on("a");
        job_c.depends_on("b");

        let jobs = vec![job_a, job_b, job_c];
        let order = topological_sort(&jobs).unwrap();

        // Order should be: a(0), b(1), c(2)
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn test_circular_dependency() {
        let mut job_a = Job::new("a");
        let mut job_b = Job::new("b");

        job_a.depends_on("b");
        job_b.depends_on("a");

        let jobs = vec![job_a, job_b];
        let result = topological_sort(&jobs);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JobError::CircularDependency(_)));
    }
}
