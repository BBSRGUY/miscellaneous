# Blang Coding Conventions

**Version**: 0.1
**Date**: 2025-01-15

This document defines the coding conventions, naming standards, and best practices for the Blang compiler codebase.

---

## Table of Contents

1. [Naming Conventions](#naming-conventions)
2. [Module Organization](#module-organization)
3. [Code Style](#code-style)
4. [Error Handling](#error-handling)
5. [Documentation](#documentation)
6. [Testing](#testing)
7. [Performance](#performance)
8. [Common Patterns](#common-patterns)

---

## Naming Conventions

### General Rules

1. **Crates**: `snake_case` with `blang_` prefix
   ```
   blang_lexer
   blang_parser
   blang_codegen_wasm
   ```

2. **Modules**: `snake_case`
   ```rust
   mod source_map;
   mod const_prop;
   mod basic_block;
   ```

3. **Types**: `PascalCase`
   ```rust
   struct Expr
   enum TokenKind
   trait Visitor
   ```

4. **Functions and Methods**: `snake_case`
   ```rust
   fn parse_expr()
   fn emit_diagnostic()
   fn lower_to_hir()
   ```

5. **Constants**: `SCREAMING_SNAKE_CASE`
   ```rust
   const MAX_LOCALS: usize = 65536;
   const DEFAULT_CAPACITY: usize = 16;
   ```

6. **Type Parameters**: Single uppercase letter or `PascalCase`
   ```rust
   struct Vec<T>
   struct HashMap<K, V>
   fn map<T, U, F: Fn(T) -> U>()

   // Or descriptive names for complex generics
   struct TypeChecker<'tcx, TyCtxt>
   ```

7. **Lifetimes**: Short descriptive names with single quote
   ```rust
   struct Parser<'a>
   struct TypeContext<'tcx>
   struct Resolver<'ast>
   ```

### Specific Naming Patterns

#### AST/IR Nodes

```rust
// Node types
struct Expr { ... }
struct Stmt { ... }
struct Item { ... }

// Node kinds (enum)
enum ExprKind { ... }
enum StmtKind { ... }
enum ItemKind { ... }

// Node IDs
struct NodeId(u32);
struct DefId(u32);
struct HirId(u32);
```

#### Visitors

```rust
// Immutable visitor
trait Visitor<'ast> {
    fn visit_expr(&mut self, expr: &'ast Expr);
    fn visit_stmt(&mut self, stmt: &'ast Stmt);
}

// Mutable visitor
trait MutVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr);
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt);
}
```

#### Contexts

```rust
// Context types
struct TypeContext { ... }   // or TyCtxt
struct InferContext { ... }   // or InferCtxt
struct CodegenContext { ... } // or CodegenCtxt

// Session/state
struct Session { ... }
struct CompilerState { ... }
```

#### Builders

```rust
struct ExprBuilder { ... }
struct MirBuilder { ... }
struct ModuleBuilder { ... }

impl ExprBuilder {
    fn new() -> Self { ... }
    fn kind(mut self, kind: ExprKind) -> Self { ... }
    fn span(mut self, span: Span) -> Self { ... }
    fn build(self) -> Expr { ... }
}
```

#### Errors

```rust
// Error types
struct ParseError { ... }
enum TypeError { ... }
struct BorrowError { ... }

// Error kinds
enum ParseErrorKind { ... }
enum TypeErrorKind { ... }

// Result type aliases
type ParseResult<T> = Result<T, ParseError>;
type TypeResult<T> = Result<T, TypeError>;
```

---

## Module Organization

### Standard Module Structure

Every crate follows this structure:

```rust
// lib.rs - Crate root, public API
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

mod internal_module;

pub mod public_module;

pub use public_module::PublicType;

// Re-export commonly used items
pub mod prelude {
    pub use crate::{Type1, Type2, function1};
}
```

### Common Module Names

```
lib.rs           # Crate root
error.rs         # Error types
visit.rs         # Visitor patterns
display.rs       # Display/Debug impls
builder.rs       # Builder patterns
util.rs          # Utilities
intern.rs        # Interning
arena.rs         # Arena allocation
index.rs         # Index types
```

### Module Organization Example

```rust
// In blang_ast/src/lib.rs
pub mod node;      // Base node types
pub mod expr;      // Expression nodes
pub mod stmt;      // Statement nodes
pub mod item;      // Item nodes
pub mod ty;        // Type nodes
pub mod pat;       // Pattern nodes
pub mod attr;      // Attributes
pub mod visit;     // Visitor trait
pub mod mut_visit; // Mutable visitor
pub mod display;   // Pretty printing

// Re-export main types
pub use node::NodeId;
pub use expr::{Expr, ExprKind};
pub use stmt::{Stmt, StmtKind};
pub use item::{Item, ItemKind};
```

---

## Code Style

### Formatting

Use `rustfmt` with project configuration:

**.rustfmt.toml**:
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### Imports

Group imports in this order:
1. Standard library
2. External crates
3. Internal crates
4. Current crate modules

```rust
// Standard library
use std::collections::HashMap;
use std::fmt;

// External crates
use anyhow::Result;
use tracing::debug;

// Internal crates
use blang_common::Symbol;
use blang_span::Span;

// Current crate
use crate::error::ParseError;
use crate::token::Token;
```

### Line Length

- Maximum line length: 100 characters
- Break long function signatures:

```rust
// Good
pub fn parse_function_declaration(
    &mut self,
    attrs: Vec<Attr>,
    visibility: Visibility,
) -> Result<Item, ParseError> {
    // ...
}

// Bad
pub fn parse_function_declaration(&mut self, attrs: Vec<Attr>, visibility: Visibility) -> Result<Item, ParseError> {
    // ...
}
```

### Whitespace

```rust
// Good
fn foo(x: i32, y: i32) -> i32 {
    x + y
}

// Bad
fn foo(x:i32,y:i32)->i32{
    x+y
}
```

### Match Expressions

```rust
// Good
match token.kind {
    TokenKind::Plus => BinOp::Add,
    TokenKind::Minus => BinOp::Sub,
    TokenKind::Star => BinOp::Mul,
    _ => return Err(error),
}

// Inline for simple cases
match token.kind {
    TokenKind::True => true,
    TokenKind::False => false,
    _ => return Err(error),
}
```

---

## Error Handling

### Use `Result` for Fallible Operations

```rust
// Good
pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
    // ...
}

// Bad - panics are only for bugs
pub fn parse_expr(&mut self) -> Expr {
    // ...
    panic!("unexpected token"); // Don't do this for expected errors
}
```

### Error Types

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("unclosed delimiter")]
    UnclosedDelimiter {
        delimiter: char,
        span: Span,
    },
}
```

### Error Context

Provide rich error context:

```rust
// Good
return Err(ParseError::UnexpectedToken {
    expected: "identifier".to_string(),
    found: format!("{:?}", token.kind),
    span: token.span,
});

// Bad
return Err(ParseError::UnexpectedToken);
```

### Use `?` Operator

```rust
// Good
let expr = self.parse_primary()?;
let stmt = self.parse_statement()?;

// Bad
let expr = match self.parse_primary() {
    Ok(e) => e,
    Err(e) => return Err(e),
};
```

---

## Documentation

### Doc Comments

Every public item must have documentation:

```rust
/// Parses an expression from the token stream.
///
/// This function implements the Pratt parsing algorithm for operator
/// precedence. It handles all expression forms including:
/// - Literals
/// - Binary operations
/// - Function calls
/// - Method calls
///
/// # Errors
///
/// Returns a `ParseError` if:
/// - An unexpected token is encountered
/// - The expression is malformed
/// - A closing delimiter is missing
///
/// # Examples
///
/// ```
/// let mut parser = Parser::new(tokens);
/// let expr = parser.parse_expr()?;
/// ```
pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
    // ...
}
```

### Module Documentation

```rust
//! Lexical analysis for Blang.
//!
//! This module provides the `Lexer` type which converts source text into
//! a stream of tokens. The lexer is implemented as an iterator and performs
//! minimal lookahead.
//!
//! # Examples
//!
//! ```
//! use blang_lexer::Lexer;
//!
//! let source = "fn main() {}";
//! let mut lexer = Lexer::new(source);
//! for token in lexer {
//!     println!("{:?}", token);
//! }
//! ```
```

### Comment Style

```rust
// Use single-line comments for brief explanations
let x = 42; // The answer

// Use multi-line comments for longer explanations
/*
 * This section implements the unification algorithm for type inference.
 * It follows the Hindley-Milner approach with extensions for:
 * - Trait bounds
 * - Lifetime parameters
 * - Associated types
 */

// Use doc comments for public API
/// Returns the type of the given expression.
pub fn type_of(&self, expr: &Expr) -> Ty {
    // ...
}
```

---

## Testing

### Unit Tests

Place unit tests in a `tests` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integer() {
        let mut lexer = Lexer::new("42");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Integer { value: 42, suffix: None });
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new(r#""hello""#);
        let token = lexer.next_token();
        assert!(matches!(token.kind, TokenKind::String { .. }));
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/parse_tests.rs
use blang_parser::Parser;
use blang_lexer::Lexer;

#[test]
fn parse_function_declaration() {
    let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let tokens: Vec<_> = Lexer::new(source).collect();
    let mut parser = Parser::new(&tokens);
    let result = parser.parse_item();
    assert!(result.is_ok());
}
```

### Snapshot Testing

Use `insta` for snapshot tests:

```rust
use insta::assert_snapshot;

#[test]
fn test_ast_display() {
    let source = "1 + 2 * 3";
    let ast = parse_expr(source).unwrap();
    assert_snapshot!(format!("{:#?}", ast));
}
```

### Property-Based Testing

Use `proptest` for property tests:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_roundtrip(s in "[a-z]+") {
        let tokens: Vec<_> = Lexer::new(&s).collect();
        let parsed = Parser::new(&tokens).parse();
        // Properties to test
    }
}
```

---

## Performance

### Avoid Unnecessary Allocations

```rust
// Good - use slices
fn process_tokens(&mut self, tokens: &[Token]) {
    // ...
}

// Bad - unnecessary clone
fn process_tokens(&mut self, tokens: Vec<Token>) {
    // Cloning entire vector
}
```

### Use `&str` Over `String`

```rust
// Good
fn parse_keyword(s: &str) -> Option<Keyword> {
    // ...
}

// Bad
fn parse_keyword(s: String) -> Option<Keyword> {
    // Unnecessarily takes ownership
}
```

### Use Arena Allocation for AST

```rust
// Good - arena-allocated
struct AstContext {
    arena: Arena<Expr>,
}

impl AstContext {
    fn alloc_expr(&self, expr: Expr) -> &Expr {
        self.arena.alloc(expr)
    }
}

// Bad - Box for every node
struct Expr {
    left: Box<Expr>,
    right: Box<Expr>,
}
```

### Use `SmallVec` for Small Collections

```rust
use smallvec::SmallVec;

// Good - stack-allocated for <= 4 items
type SmallArgs = SmallVec<[Expr; 4]>;

fn parse_args(&mut self) -> SmallArgs {
    // Most function calls have few arguments
}
```

### Profile Before Optimizing

```rust
// Use tracing for performance profiling
use tracing::instrument;

#[instrument(skip(self))]
fn expensive_operation(&mut self) {
    // Automatically logged with timing info
}
```

---

## Common Patterns

### Newtype Pattern

```rust
// Strong typing with zero cost
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        NodeId(id)
    }
}
```

### Builder Pattern

```rust
pub struct ExprBuilder {
    kind: Option<ExprKind>,
    span: Option<Span>,
}

impl ExprBuilder {
    pub fn new() -> Self {
        Self { kind: None, span: None }
    }

    pub fn kind(mut self, kind: ExprKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn build(self) -> Expr {
        Expr {
            kind: self.kind.expect("kind is required"),
            span: self.span.expect("span is required"),
            id: NodeId::new(),
        }
    }
}
```

### Visitor Pattern

```rust
pub trait Visitor<'ast> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        walk_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        walk_stmt(self, stmt);
    }
}

// Default traversal
pub fn walk_expr<'ast, V: Visitor<'ast>>(visitor: &mut V, expr: &'ast Expr) {
    match &expr.kind {
        ExprKind::Binary(_, left, right) => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        // ... other cases
    }
}
```

### Index Types with `IndexVec`

```rust
use blang_common::index::IndexVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(u32);

impl From<u32> for LocalId {
    fn from(id: u32) -> Self {
        LocalId(id)
    }
}

impl From<LocalId> for usize {
    fn from(id: LocalId) -> usize {
        id.0 as usize
    }
}

// Type-safe indexing
pub struct LocalTable {
    locals: IndexVec<LocalId, Local>,
}
```

### Interned Strings

```rust
use blang_common::symbol::{Symbol, Interner};

pub struct Compiler {
    interner: Interner,
}

impl Compiler {
    pub fn intern(&mut self, s: &str) -> Symbol {
        self.interner.intern(s)
    }

    pub fn resolve(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym)
    }
}
```

### Type State Pattern

```rust
// Use phantom types to enforce state transitions
struct Parser<State> {
    tokens: Vec<Token>,
    pos: usize,
    _state: PhantomData<State>,
}

struct Initialized;
struct Parsing;
struct Finished;

impl Parser<Initialized> {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            _state: PhantomData,
        }
    }

    pub fn begin(self) -> Parser<Parsing> {
        Parser {
            tokens: self.tokens,
            pos: self.pos,
            _state: PhantomData,
        }
    }
}

impl Parser<Parsing> {
    pub fn parse_expr(&mut self) -> Result<Expr, Error> {
        // ...
    }

    pub fn finish(self) -> Parser<Finished> {
        Parser {
            tokens: self.tokens,
            pos: self.pos,
            _state: PhantomData,
        }
    }
}
```

---

## Clippy Configuration

**.clippy.toml**:
```toml
# Deny
avoid-breaking-exported-api = true
warn-on-all-wildcard-imports = true

# Warn
cognitive-complexity-threshold = 50
too-many-arguments-threshold = 8
```

**Lint levels** (in lib.rs):
```rust
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
)]
#![allow(
    clippy::missing_errors_doc,      // We document errors in other ways
    clippy::module_name_repetitions,  // Common in compiler code
)]
```

---

## Summary

Following these conventions ensures:

1. **Consistency**: Code looks uniform across the codebase
2. **Readability**: Easy to understand and navigate
3. **Maintainability**: Changes are localized and safe
4. **Performance**: Efficient patterns are used by default
5. **Quality**: High code quality through linting and testing

All contributors should follow these conventions. Use `rustfmt` and `clippy` to automatically enforce many of these rules.

For questions or clarifications, refer to:
- [The Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md)
- Existing codebase as examples
