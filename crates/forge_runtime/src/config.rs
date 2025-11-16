//! # Configuration Module
//!
//! Provides configuration management for the Forge platform.
//! Supports loading from YAML files and environment variables.

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::Level;

/// Errors that can occur during configuration operations.
#[derive(Debug, Error)]
pub enum ForgeConfigError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ForgeConfigError>;

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// API server configuration
    pub api: ApiConfig,

    /// Database configuration
    pub db: DbConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Inference configuration
    pub inference: InferenceConfig,

    /// GPU configuration
    pub gpu: GpuConfig,
}

/// API server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Host to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// Enable CORS
    pub cors_enabled: bool,

    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// Database file path
    pub path: PathBuf,

    /// Enable WAL mode
    pub wal_mode: bool,

    /// Connection pool size
    pub pool_size: u32,

    /// Enable query logging
    pub log_queries: bool,
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base data directory
    pub data_dir: PathBuf,

    /// Models directory
    pub models_dir: PathBuf,

    /// Datasets directory
    pub datasets_dir: PathBuf,

    /// Checkpoints directory
    pub checkpoints_dir: PathBuf,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, pretty, compact)
    pub format: LogFormat,

    /// Enable file logging
    pub file_enabled: bool,

    /// Log file path
    pub file_path: PathBuf,

    /// Enable console logging
    pub console_enabled: bool,
}

/// Log output format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON structured logs
    Json,
    /// Pretty human-readable logs
    Pretty,
    /// Compact logs
    Compact,
}

/// Inference configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Maximum concurrent inference tasks
    pub max_concurrent_tasks: usize,

    /// Default temperature for completions
    pub default_temperature: f32,

    /// Default max tokens
    pub default_max_tokens: usize,

    /// Enable model caching
    pub enable_model_cache: bool,
}

/// GPU configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Enable GPU acceleration
    pub enabled: bool,

    /// Prefer high performance GPU
    pub high_performance: bool,

    /// Enable WebGPU
    pub webgpu_enabled: bool,
}

impl AppConfig {
    /// Load configuration from files and environment variables.
    ///
    /// Configuration is loaded in the following order (later sources override earlier):
    /// 1. configs/default.yaml
    /// 2. configs/local.yaml (if exists)
    /// 3. Environment variables with FORGE__ prefix
    pub fn load() -> Result<Self> {
        Self::load_from("configs")
    }

    /// Load configuration from a specific directory.
    pub fn load_from<P: AsRef<Path>>(config_dir: P) -> Result<Self> {
        let config_dir = config_dir.as_ref();

        let builder = Config::builder()
            // Load default config
            .add_source(File::from(config_dir.join("default.yaml")).required(true))
            // Load local config (optional)
            .add_source(File::from(config_dir.join("local.yaml")).required(false))
            // Add environment variables with FORGE__ prefix
            // e.g., FORGE__API__PORT=3001
            .add_source(
                Environment::with_prefix("FORGE")
                    .separator("__")
                    .try_parsing(true),
            );

        let config = builder.build()?;
        let app_config: AppConfig = config.try_deserialize()?;

        // Validate configuration
        app_config.validate()?;

        Ok(app_config)
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<()> {
        // Validate port range
        if self.api.port == 0 {
            return Err(ForgeConfigError::InvalidConfig(
                "API port must be non-zero".to_string(),
            ));
        }

        // Validate log level
        let _ = self.logging.log_level().map_err(|e| {
            ForgeConfigError::InvalidConfig(format!("Invalid log level: {}", e))
        })?;

        // Validate temperature range
        if !(0.0..=2.0).contains(&self.inference.default_temperature) {
            return Err(ForgeConfigError::InvalidConfig(
                "Temperature must be between 0.0 and 2.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the full path to the database file.
    pub fn db_path(&self) -> PathBuf {
        if self.db.path.is_absolute() {
            self.db.path.clone()
        } else {
            self.storage.data_dir.join(&self.db.path)
        }
    }

    /// Get the full path to the models directory.
    pub fn models_path(&self) -> PathBuf {
        if self.storage.models_dir.is_absolute() {
            self.storage.models_dir.clone()
        } else {
            self.storage.data_dir.join(&self.storage.models_dir)
        }
    }

    /// Get the full path to the datasets directory.
    pub fn datasets_path(&self) -> PathBuf {
        if self.storage.datasets_dir.is_absolute() {
            self.storage.datasets_dir.clone()
        } else {
            self.storage.data_dir.join(&self.storage.datasets_dir)
        }
    }
}

impl LoggingConfig {
    /// Parse the log level string into a tracing Level.
    pub fn log_level(&self) -> std::result::Result<Level, String> {
        match self.level.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            _ => Err(format!("Invalid log level: {}", self.level)),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            db: DbConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
            inference: InferenceConfig::default(),
            gpu: GpuConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors_enabled: true,
            timeout_seconds: 30,
        }
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("forge.db"),
            wal_mode: true,
            pool_size: 5,
            log_queries: false,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            models_dir: PathBuf::from("models"),
            datasets_dir: PathBuf::from("datasets"),
            checkpoints_dir: PathBuf::from("checkpoints"),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            file_enabled: false,
            file_path: PathBuf::from("logs/forge.log"),
            console_enabled: true,
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            default_temperature: 0.7,
            default_max_tokens: 2048,
            enable_model_cache: true,
        }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            high_performance: true,
            webgpu_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.api.port, 3000);
        assert_eq!(config.logging.level, "info");
        assert!(config.gpu.enabled);
    }

    #[test]
    fn test_log_level_parsing() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            ..Default::default()
        };
        assert_eq!(config.log_level().unwrap(), Level::DEBUG);
    }

    #[test]
    fn test_path_resolution() {
        let config = AppConfig::default();
        let db_path = config.db_path();
        assert!(db_path.to_str().unwrap().contains("forge.db"));
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.api.port = 0;
        assert!(config.validate().is_err());
    }
}
