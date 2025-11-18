//! Training and finetuning support for ML models.
//!
//! This module provides APIs for training and finetuning models,
//! including LoRA/QLoRA support.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

pub mod trainer;

/// Errors that can occur during training.
#[derive(Debug, Error)]
pub enum TrainingError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Dataset error: {0}")]
    DatasetError(String),

    #[error("Training failed: {0}")]
    TrainingFailed(String),

    #[error("Checkpoint error: {0}")]
    CheckpointError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TrainingError>;

/// Training method/approach.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrainingMethod {
    /// Full model finetuning
    FullFinetune,
    /// Low-Rank Adaptation (LoRA)
    LoRA,
    /// Quantized LoRA
    QLoRA,
}

/// Optimizer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizerConfig {
    /// AdamW optimizer
    AdamW {
        learning_rate: f32,
        weight_decay: f32,
        beta1: f32,
        beta2: f32,
    },
    /// SGD optimizer
    SGD {
        learning_rate: f32,
        momentum: f32,
    },
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self::AdamW {
            learning_rate: 2e-4,
            weight_decay: 0.01,
            beta1: 0.9,
            beta2: 0.999,
        }
    }
}

/// Learning rate scheduler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LRSchedulerConfig {
    /// Constant learning rate
    Constant,
    /// Linear warmup with cosine decay
    CosineWithWarmup { warmup_steps: usize },
    /// Linear decay
    Linear { total_steps: usize },
}

impl Default for LRSchedulerConfig {
    fn default() -> Self {
        Self::CosineWithWarmup { warmup_steps: 100 }
    }
}

/// LoRA-specific hyperparameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoRAConfig {
    /// Rank of LoRA adaptation
    pub rank: usize,
    /// Alpha parameter for LoRA scaling
    pub alpha: f32,
    /// Dropout rate for LoRA layers
    pub dropout: f32,
    /// Target modules to apply LoRA (e.g., ["q_proj", "v_proj"])
    pub target_modules: Vec<String>,
}

impl Default for LoRAConfig {
    fn default() -> Self {
        Self {
            rank: 8,
            alpha: 16.0,
            dropout: 0.1,
            target_modules: vec!["q_proj".to_string(), "v_proj".to_string()],
        }
    }
}

/// Training configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    /// Base model ID to finetune
    pub base_model_id: String,

    /// Path to training dataset
    pub dataset_path: PathBuf,

    /// Training method
    pub method: TrainingMethod,

    /// Number of training epochs
    pub num_epochs: usize,

    /// Batch size
    pub batch_size: usize,

    /// Maximum sequence length
    pub max_seq_length: usize,

    /// Gradient accumulation steps
    pub gradient_accumulation_steps: usize,

    /// Optimizer configuration
    pub optimizer: OptimizerConfig,

    /// Learning rate scheduler
    pub lr_scheduler: LRSchedulerConfig,

    /// LoRA configuration (if method is LoRA/QLoRA)
    pub lora_config: Option<LoRAConfig>,

    /// Output directory for checkpoints
    pub output_dir: PathBuf,

    /// Save checkpoint every N steps
    pub save_steps: usize,

    /// Logging interval (steps)
    pub logging_steps: usize,

    /// Evaluation interval (steps)
    pub eval_steps: Option<usize>,

    /// Maximum number of steps (overrides epochs if set)
    pub max_steps: Option<usize>,

    /// Random seed
    pub seed: u64,

    /// Use mixed precision (fp16)
    pub fp16: bool,

    /// Use gradient checkpointing
    pub gradient_checkpointing: bool,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            base_model_id: String::new(),
            dataset_path: PathBuf::new(),
            method: TrainingMethod::LoRA,
            num_epochs: 3,
            batch_size: 4,
            max_seq_length: 512,
            gradient_accumulation_steps: 1,
            optimizer: OptimizerConfig::default(),
            lr_scheduler: LRSchedulerConfig::default(),
            lora_config: Some(LoRAConfig::default()),
            output_dir: PathBuf::from("./checkpoints"),
            save_steps: 500,
            logging_steps: 10,
            eval_steps: Some(100),
            max_steps: None,
            seed: 42,
            fp16: false,
            gradient_checkpointing: false,
        }
    }
}

impl TrainConfig {
    /// Validate the training configuration.
    pub fn validate(&self) -> Result<()> {
        if self.base_model_id.is_empty() {
            return Err(TrainingError::InvalidConfig(
                "base_model_id is required".to_string(),
            ));
        }

        if !self.dataset_path.exists() {
            return Err(TrainingError::DatasetError(format!(
                "Dataset path does not exist: {}",
                self.dataset_path.display()
            )));
        }

        if self.batch_size == 0 {
            return Err(TrainingError::InvalidConfig(
                "batch_size must be > 0".to_string(),
            ));
        }

        if self.num_epochs == 0 && self.max_steps.is_none() {
            return Err(TrainingError::InvalidConfig(
                "Either num_epochs or max_steps must be > 0".to_string(),
            ));
        }

        if matches!(self.method, TrainingMethod::LoRA | TrainingMethod::QLoRA)
            && self.lora_config.is_none()
        {
            return Err(TrainingError::InvalidConfig(
                "lora_config required for LoRA/QLoRA training".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate total training steps.
    pub fn total_steps(&self, dataset_size: usize) -> usize {
        if let Some(max_steps) = self.max_steps {
            max_steps
        } else {
            let steps_per_epoch =
                (dataset_size + self.batch_size - 1) / self.batch_size / self.gradient_accumulation_steps;
            steps_per_epoch * self.num_epochs
        }
    }
}

/// Training metrics at a specific step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    /// Current step
    pub step: usize,
    /// Current epoch
    pub epoch: f32,
    /// Training loss
    pub loss: f32,
    /// Learning rate
    pub learning_rate: f32,
    /// Gradient norm (if available)
    pub grad_norm: Option<f32>,
    /// Evaluation loss (if available)
    pub eval_loss: Option<f32>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Training state/progress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrainingState {
    /// Training is initializing
    Initializing,
    /// Training is running
    Running,
    /// Training is paused
    Paused,
    /// Training completed successfully
    Completed,
    /// Training failed
    Failed,
    /// Training was cancelled
    Cancelled,
}

/// Training job information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    /// Unique job ID
    pub id: String,
    /// Training configuration
    pub config: TrainConfig,
    /// Current state
    pub state: TrainingState,
    /// Current step
    pub current_step: usize,
    /// Total steps
    pub total_steps: usize,
    /// Latest metrics
    pub latest_metrics: Option<TrainingMetrics>,
    /// All metrics history
    pub metrics_history: Vec<TrainingMetrics>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Started timestamp
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completed timestamp
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TrainingJob {
    /// Create a new training job.
    pub fn new(id: String, config: TrainConfig, total_steps: usize) -> Self {
        Self {
            id,
            config,
            state: TrainingState::Initializing,
            current_step: 0,
            total_steps,
            latest_metrics: None,
            metrics_history: Vec::new(),
            error: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Update job with new metrics.
    pub fn update_metrics(&mut self, metrics: TrainingMetrics) {
        self.current_step = metrics.step;
        self.latest_metrics = Some(metrics.clone());
        self.metrics_history.push(metrics);

        // Keep only recent metrics to avoid unbounded growth
        if self.metrics_history.len() > 1000 {
            self.metrics_history.drain(0..500);
        }
    }

    /// Get progress percentage.
    pub fn progress(&self) -> f32 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.current_step as f32 / self.total_steps as f32) * 100.0
        }
    }

    /// Mark job as started.
    pub fn mark_started(&mut self) {
        self.state = TrainingState::Running;
        self.started_at = Some(chrono::Utc::now());
    }

    /// Mark job as completed.
    pub fn mark_completed(&mut self) {
        self.state = TrainingState::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark job as failed with error message.
    pub fn mark_failed(&mut self, error: String) {
        self.state = TrainingState::Failed;
        self.error = Some(error);
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark job as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.state = TrainingState::Cancelled;
        self.completed_at = Some(chrono::Utc::now());
    }
}
