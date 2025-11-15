//! Source code location tracking for the Blang compiler.
//!
//! This crate provides types for tracking positions and spans in source code,
//! essential for error reporting and diagnostics.
//!
//! # Core Types
//!
//! - [`BytePos`]: A byte offset in a source file
//! - [`Span`]: A range of source code (start and end positions)
//! - [`SourceFile`]: Representation of a single source file
//! - [`SourceMap`]: Global registry of all source files
//!
//! # Example
//!
//! ```
//! use blang_span::{BytePos, Span};
//!
//! let start = BytePos::from(0);
//! let end = BytePos::from(10);
//! let span = Span::new(start, end);
//!
//! assert_eq!(span.len(), 10);
//! ```

pub mod pos;
pub mod source_file;
pub mod source_map;
pub mod span;

pub use pos::{BytePos, CharPos, LineCol};
pub use source_file::SourceFile;
pub use source_map::SourceMap;
pub use span::Span;

/// A dummy span for testing or when a span is not available.
pub const DUMMY_SPAN: Span = Span::DUMMY;