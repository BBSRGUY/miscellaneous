//! Step execution primitives

use crate::error::StepResult;
use std::collections::HashMap;

/// A step function that can be executed
pub enum StepFn {
    /// Synchronous step function
    Sync(Box<dyn FnMut() -> StepResult + Send>),

    /// Step with access to context
    WithContext(Box<dyn FnMut(&mut StepContext) -> StepResult + Send>),
}

/// Context available to steps during execution
pub struct StepContext {
    /// Environment variables
    pub env: HashMap<String, String>,

    /// Step-specific data
    pub data: HashMap<String, String>,

    /// Job name
    pub job_name: String,

    /// Step name
    pub step_name: String,
}

impl StepContext {
    /// Create a new step context
    pub fn new(job_name: String, step_name: String) -> Self {
        Self {
            env: HashMap::new(),
            data: HashMap::new(),
            job_name,
            step_name,
        }
    }

    /// Set an environment variable
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }

    /// Get an environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.env.get(key)
    }

    /// Set step data
    pub fn set_data(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get step data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// A step in a job
pub struct Step {
    /// Step name
    pub name: String,

    /// Step function
    pub func: StepFn,

    /// Configuration options
    pub config: HashMap<String, String>,

    /// Whether this step can run in parallel with others
    pub parallel: bool,

    /// Whether to continue on error
    pub continue_on_error: bool,
}

impl Step {
    /// Create a new step
    pub fn new(name: impl Into<String>, func: StepFn) -> Self {
        Self {
            name: name.into(),
            func,
            config: HashMap::new(),
            parallel: false,
            continue_on_error: false,
        }
    }

    /// Set parallel execution flag
    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set continue on error flag
    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Add a configuration option
    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Execute the step
    pub fn execute(&mut self, context: &mut StepContext) -> StepResult {
        match &mut self.func {
            StepFn::Sync(f) => f(),
            StepFn::WithContext(f) => f(context),
        }
    }
}
