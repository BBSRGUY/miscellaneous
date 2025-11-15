//! Lexical analysis for the Blang programming language.
//!
//! This crate provides a lexer that converts UTF-8 source text into a stream
//! of tokens. The lexer is implemented as an iterator and supports all Blang
//! language constructs including:
//!
//! - Keywords (fn, let, component, script, module, etc.)
//! - Literals (integers, floats, strings, characters, template strings)
//! - Operators (+, -, *, /, ==, !=, etc.)
//! - Punctuation ((, ), {, }, ;, :, etc.)
//! - Comments (line and block, including nested block comments)
//!
//! # Examples
//!
//! Basic lexing:
//!
//! ```
//! use blang_lexer::Lexer;
//!
//! let source = "fn main() { let x = 42; }";
//! let mut lexer = Lexer::new(source);
//!
//! // Collect all tokens
//! let tokens: Vec<_> = lexer.collect();
//! assert!(tokens.len() > 0);
//! ```
//!
//! Examining individual tokens:
//!
//! ```
//! use blang_lexer::{Lexer, TokenKind};
//!
//! let source = "let x = 42;";
//! let mut lexer = Lexer::new(source);
//!
//! let token = lexer.next_token();
//! assert_eq!(token.kind, TokenKind::Let);
//! ```

#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

mod cursor;
mod lexer;
mod token;
mod unescape;

pub use lexer::Lexer;
pub use token::{Base, Token, TokenKind};
pub use unescape::{unescape, UnescapeCallback, UnescapeError, UnescapeMode};
