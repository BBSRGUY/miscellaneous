//! # Logging Module
//!
//! Provides logging setup and utilities for the Forge platform.

use crate::config::{LogFormat, LoggingConfig};
use std::io;
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Errors that can occur during logging setup.
#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("Failed to initialize logging: {0}")]
    InitError(String),

    #[error("Invalid log level: {0}")]
    InvalidLevel(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, LoggingError>;

/// Initialize the logging system based on configuration.
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let log_level = config
        .log_level()
        .map_err(|e| LoggingError::InvalidLevel(e))?;

    // Create the base filter
    let env_filter = EnvFilter::builder()
        .with_default_directive(level_to_filter(log_level).into())
        .from_env_lossy();

    // Build the subscriber with the appropriate format
    let subscriber = tracing_subscriber::registry().with(env_filter);

    // Add console layer if enabled
    if config.console_enabled {
        let console_layer = match config.format {
            LogFormat::Json => fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(true)
                .with_span_list(true)
                .with_writer(io::stderr)
                .boxed(),
            LogFormat::Pretty => fmt::layer()
                .pretty()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .with_writer(io::stderr)
                .boxed(),
            LogFormat::Compact => fmt::layer()
                .compact()
                .with_target(false)
                .with_thread_ids(false)
                .with_writer(io::stderr)
                .boxed(),
        };

        subscriber.with(console_layer).init();
    } else {
        // File-only logging
        if config.file_enabled {
            return Err(LoggingError::InitError(
                "File-only logging not yet implemented".to_string(),
            ));
        } else {
            // No logging enabled
            subscriber.init();
        }
    }

    Ok(())
}

/// Initialize logging with defaults (for quick setup).
pub fn init_default_logging() -> Result<()> {
    let config = LoggingConfig::default();
    init_logging(&config)
}

/// Initialize logging with a specific log level.
pub fn init_with_level(level: Level) -> Result<()> {
    let config = LoggingConfig {
        level: level_to_string(level),
        ..Default::default()
    };
    init_logging(&config)
}

/// Convert a tracing Level to LevelFilter.
fn level_to_filter(level: Level) -> LevelFilter {
    match level {
        Level::TRACE => LevelFilter::TRACE,
        Level::DEBUG => LevelFilter::DEBUG,
        Level::INFO => LevelFilter::INFO,
        Level::WARN => LevelFilter::WARN,
        Level::ERROR => LevelFilter::ERROR,
    }
}

/// Convert a tracing Level to a string.
fn level_to_string(level: Level) -> String {
    match level {
        Level::TRACE => "trace".to_string(),
        Level::DEBUG => "debug".to_string(),
        Level::INFO => "info".to_string(),
        Level::WARN => "warn".to_string(),
        Level::ERROR => "error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_conversion() {
        assert_eq!(level_to_string(Level::INFO), "info");
        assert_eq!(level_to_string(Level::DEBUG), "debug");
    }

    #[test]
    fn test_level_filter_conversion() {
        assert_eq!(level_to_filter(Level::INFO), LevelFilter::INFO);
        assert_eq!(level_to_filter(Level::DEBUG), LevelFilter::DEBUG);
    }
}
