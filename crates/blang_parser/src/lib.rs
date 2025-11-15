//! Parser for the Blang programming language.
//!
//! This crate provides a complete parser that converts source code into an
//! Abstract Syntax Tree (AST). The parser uses a combination of:
//!
//! - **Recursive descent** parsing for most language constructs
//! - **Pratt parsing** for expression precedence and associativity
//!
//! # Example
//!
//! ```
//! use blang_parser::parse;
//!
//! let source = r#"
//! fn add(a: i32, b: i32) -> i32 {
//!     a + b
//! }
//! "#;
//!
//! match parse(source) {
//!     Ok(items) => println!("Parsed {} items", items.len()),
//!     Err(err) => eprintln!("Parse error: {}", err),
//! }
//! ```
//!
//! # Error Handling
//!
//! The parser provides detailed error messages with source locations:
//!
//! - Unexpected token errors show what was found vs. expected
//! - Missing token errors indicate required punctuation
//! - Span information for precise error reporting
//!
//! # Architecture
//!
//! - [`parser`]: Main parser implementation
//! - [`error`]: Error types and reporting
//! - [`token_stream`]: Token iteration with lookahead
//! - [`ty`], [`pat`], [`expr`], [`stmt`], [`item`]: Parsing modules

pub mod error;
pub mod expr;
pub mod expr_helpers;
pub mod item;
pub mod parser;
pub mod pat;
pub mod stmt;
pub mod token_stream;
pub mod ty;
pub mod utils;

pub use error::{ParseError, ParseErrorKind, ParseResult};
pub use parser::Parser;

use blang_ast::Item;

/// Parse Blang source code into a list of items.
///
/// This is the main entry point for parsing Blang programs.
///
/// # Example
///
/// ```
/// use blang_parser::parse;
///
/// let source = "fn main() {}";
/// let items = parse(source).unwrap();
/// assert_eq!(items.len(), 1);
/// ```
///
/// # Errors
///
/// Returns a [`ParseError`] if the source code contains syntax errors.
pub fn parse(source: &str) -> ParseResult<Vec<Item>> {
    let mut parser = Parser::new(source);
    let mut items = Vec::new();

    while !parser.stream.at_eof() {
        items.push(parser.parse_item()?);
    }

    Ok(items)
}

/// Parse a single expression from source code.
///
/// This is useful for testing or parsing expression fragments.
///
/// # Example
///
/// ```
/// use blang_parser::parse_expr;
///
/// let source = "1 + 2 * 3";
/// let expr = parse_expr(source).unwrap();
/// ```
pub fn parse_expr(source: &str) -> ParseResult<blang_ast::Expr> {
    let mut parser = Parser::new(source);
    parser.parse_expr()
}

/// Parse a single statement from source code.
///
/// # Example
///
/// ```
/// use blang_parser::parse_stmt;
///
/// let source = "let x = 42;";
/// let stmt = parse_stmt(source).unwrap();
/// ```
pub fn parse_stmt(source: &str) -> ParseResult<blang_ast::Stmt> {
    let mut parser = Parser::new(source);
    parser.parse_stmt()
}

/// Parse a single type expression from source code.
///
/// # Example
///
/// ```
/// use blang_parser::parse_type;
///
/// let source = "Vec<i32>";
/// let ty = parse_type(source).unwrap();
/// ```
pub fn parse_type(source: &str) -> ParseResult<blang_ast::Ty> {
    let mut parser = Parser::new(source);
    parser.parse_ty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let items = parse("").unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_parse_simple_function() {
        let source = "fn foo() {}";
        let items = parse(source).unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_expr() {
        let source = "1 + 2";
        let expr = parse_expr(source).unwrap();
        assert!(matches!(expr.kind, blang_ast::ExprKind::Binary(_)));
    }

    #[test]
    fn test_parse_stmt() {
        let source = "let x = 5;";
        let stmt = parse_stmt(source).unwrap();
        assert!(matches!(stmt.kind, blang_ast::StmtKind::Let(_)));
    }

    #[test]
    fn test_parse_type() {
        let source = "i32";
        let ty = parse_type(source).unwrap();
        assert!(matches!(ty.kind, blang_ast::TyKind::Primitive(_)));
    }

    #[test]
    fn test_parse_error() {
        let source = "fn {"; // Invalid syntax
        let result = parse(source);
        assert!(result.is_err());
    }
}
