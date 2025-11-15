//! Error types for the Blang CLI

use std::path::PathBuf;
use thiserror::Error;

/// CLI-specific errors
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Failed to read source file '{path}': {source}")]
    ReadSource {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to write output file '{path}': {source}")]
    WriteOutput {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Parse error at {location}: {message}")]
    Parse { location: String, message: String },

    #[error("Type error: {0}")]
    Type(String),

    #[error("IR lowering error: {0}")]
    IrLowering(String),

    #[error("Code generation error: {0}")]
    Codegen(String),

    #[error("Invalid optimization level: {0} (must be 0-3)")]
    InvalidOptLevel(u8),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Failed to create directory '{path}': {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Dev server error: {0}")]
    DevServer(String),
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;
