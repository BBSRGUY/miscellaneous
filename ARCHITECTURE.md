# Blang Project Architecture

**Version**: 0.1
**Date**: 2025-01-15
**Status**: Design Phase

---

## Table of Contents

1. [Overview](#overview)
2. [Repository Structure](#repository-structure)
3. [Crate Organization](#crate-organization)
4. [Crate Responsibilities](#crate-responsibilities)
5. [Dependency Graph](#dependency-graph)
6. [Module Naming Conventions](#module-naming-conventions)
7. [API Design Principles](#api-design-principles)
8. [Testing Strategy](#testing-strategy)

---

## Overview

The Blang compiler is organized as a Cargo workspace with 18 focused crates, each with clear responsibilities and minimal coupling. The architecture follows a traditional compiler pipeline with clean phase separation.

### Design Principles

1. **Separation of Concerns**: Each crate has a single, well-defined responsibility
2. **Minimal Coupling**: Crates depend only on what they need
3. **Testability**: Each crate can be tested in isolation
4. **Incremental Compilation**: Cargo can rebuild only changed crates
5. **Clear Data Flow**: Information flows unidirectionally through the pipeline

### Core Architecture

```
Source Files (.blang, .bl, .bs)
    ↓
[blang_lexer] → Tokens
    ↓
[blang_parser] → AST (blang_ast)
    ↓
[blang_resolve] → Resolved AST
    ↓
[blang_ir] → HIR
    ↓
[blang_typeck] → Typed HIR
    ↓
[blang_ir] → MIR
    ↓
[blang_borrow] → Validated MIR
    ↓
[blang_optimize] → Optimized MIR
    ↓
[blang_ir] → LIR
    ↓
[blang_codegen_wasm] → WASM
[blang_codegen_js] → JS Glue
    ↓
Output Bundle
```

---

## Repository Structure

```
blang/
├── Cargo.toml                          # Workspace root
├── Cargo.lock                          # Dependency lock file
├── README.md                           # Project overview
├── LANGUAGE_SPEC.md                    # Language specification
├── GRAMMAR.md                          # Grammar specification
├── ROADMAP.md                          # Development roadmap
├── PROJECT_STRUCTURE.md                # Original structure doc
├── ARCHITECTURE.md                     # This file
├── CONTRIBUTING.md                     # Contribution guidelines
├── LICENSE-MIT                         # MIT license
├── LICENSE-APACHE                      # Apache 2.0 license
├── .gitignore                          # Git ignore rules
├── .rustfmt.toml                       # Rust formatter config
├── .clippy.toml                        # Clippy linter config
│
├── .github/                            # GitHub configuration
│   ├── workflows/
│   │   ├── ci.yml                      # Continuous integration
│   │   ├── release.yml                 # Release automation
│   │   └── docs.yml                    # Documentation build
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
│
├── crates/                             # All Rust crates
│   │
│   ├── blang_common/                   # Shared utilities
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── arena.rs                # Arena allocator
│   │   │   ├── symbol.rs               # String interning
│   │   │   ├── index.rs                # Index types
│   │   │   └── collections.rs          # Fast hash maps/sets
│   │   └── tests/
│   │
│   ├── blang_span/                     # Source location tracking
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── span.rs                 # Span type
│   │   │   ├── source_map.rs           # Source file management
│   │   │   └── pos.rs                  # Position types
│   │   └── tests/
│   │
│   ├── blang_diagnostics/              # Error reporting
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── diagnostic.rs           # Diagnostic types
│   │   │   ├── emitter.rs              # Output formatting
│   │   │   ├── snippet.rs              # Code snippets
│   │   │   └── codes.rs                # Error codes
│   │   └── tests/
│   │
│   ├── blang_lexer/                    # Lexical analysis
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lexer.rs                # Main lexer
│   │   │   ├── token.rs                # Token types
│   │   │   ├── cursor.rs               # Character iterator
│   │   │   └── unescape.rs             # String unescaping
│   │   └── tests/
│   │       ├── literals.rs
│   │       ├── keywords.rs
│   │       └── operators.rs
│   │
│   ├── blang_ast/                      # Abstract syntax tree
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── node.rs                 # Base AST node
│   │   │   ├── expr.rs                 # Expression nodes
│   │   │   ├── stmt.rs                 # Statement nodes
│   │   │   ├── item.rs                 # Item nodes
│   │   │   ├── ty.rs                   # Type nodes
│   │   │   ├── pat.rs                  # Pattern nodes
│   │   │   ├── attr.rs                 # Attributes
│   │   │   ├── visit.rs                # Visitor pattern
│   │   │   ├── mut_visit.rs            # Mutable visitor
│   │   │   └── display.rs              # Pretty printing
│   │   └── tests/
│   │
│   ├── blang_parser/                   # Parser
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs               # Main parser
│   │   │   ├── expr.rs                 # Expression parsing
│   │   │   ├── stmt.rs                 # Statement parsing
│   │   │   ├── item.rs                 # Item parsing
│   │   │   ├── ty.rs                   # Type parsing
│   │   │   ├── pat.rs                  # Pattern parsing
│   │   │   └── recovery.rs             # Error recovery
│   │   └── tests/
│   │       ├── expressions.rs
│   │       ├── statements.rs
│   │       ├── functions.rs
│   │       └── error_recovery.rs
│   │
│   ├── blang_resolve/                  # Name resolution
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── resolve.rs              # Main resolver
│   │   │   ├── scope.rs                # Scope management
│   │   │   ├── import.rs               # Import resolution
│   │   │   ├── def.rs                  # Definition tracking
│   │   │   └── path.rs                 # Path resolution
│   │   └── tests/
│   │
│   ├── blang_types/                    # Type representation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ty.rs                   # Type representation
│   │   │   ├── subst.rs                # Type substitution
│   │   │   ├── traits.rs               # Trait definitions
│   │   │   └── layout.rs               # Memory layout
│   │   └── tests/
│   │
│   ├── blang_typeck/                   # Type checking
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── check.rs                # Type checker
│   │   │   ├── infer.rs                # Type inference
│   │   │   ├── unify.rs                # Unification
│   │   │   ├── method.rs               # Method resolution
│   │   │   └── coherence.rs            # Trait coherence
│   │   └── tests/
│   │
│   ├── blang_ir/                       # Intermediate representations
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── hir/                    # High-level IR
│   │   │   │   ├── mod.rs
│   │   │   │   ├── lower.rs            # AST → HIR
│   │   │   │   ├── node.rs
│   │   │   │   └── visit.rs
│   │   │   ├── mir/                    # Mid-level IR
│   │   │   │   ├── mod.rs
│   │   │   │   ├── build.rs            # HIR → MIR
│   │   │   │   ├── basic_block.rs
│   │   │   │   └── visit.rs
│   │   │   └── lir/                    # Low-level IR
│   │   │       ├── mod.rs
│   │   │       ├── lower.rs            # MIR → LIR
│   │   │       └── inst.rs
│   │   └── tests/
│   │
│   ├── blang_borrow/                   # Borrow checker
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── check.rs                # Borrow checking
│   │   │   ├── lifetime.rs             # Lifetime inference
│   │   │   └── region.rs               # Region analysis
│   │   └── tests/
│   │
│   ├── blang_optimize/                 # Optimization passes
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pass_manager.rs         # Pass orchestration
│   │   │   ├── inline.rs               # Function inlining
│   │   │   ├── const_prop.rs           # Constant propagation
│   │   │   ├── dce.rs                  # Dead code elimination
│   │   │   └── simplify_cfg.rs         # CFG simplification
│   │   └── tests/
│   │
│   ├── blang_codegen_wasm/             # WASM code generation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── codegen.rs              # Main codegen
│   │   │   ├── module.rs               # WASM module builder
│   │   │   ├── function.rs             # Function codegen
│   │   │   ├── expr.rs                 # Expression codegen
│   │   │   ├── memory.rs               # Memory management
│   │   │   └── abi.rs                  # Calling convention
│   │   └── tests/
│   │
│   ├── blang_codegen_js/               # JS glue generation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── codegen.rs              # JS codegen
│   │   │   ├── bindings.rs             # WASM bindings
│   │   │   ├── dom.rs                  # DOM bindings
│   │   │   └── module.rs               # ES module gen
│   │   └── tests/
│   │
│   ├── blang_component/                # Component mode compiler
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parse.rs                # Component parsing
│   │   │   ├── view.rs                 # View compilation
│   │   │   ├── style.rs                # Style compilation
│   │   │   ├── reactive.rs             # Reactivity codegen
│   │   │   └── lifecycle.rs            # Lifecycle hooks
│   │   └── tests/
│   │
│   ├── blang_script/                   # Script mode compiler
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parse.rs                # Script parsing
│   │   │   ├── job.rs                  # Job compilation
│   │   │   ├── schedule.rs             # Dependency resolution
│   │   │   └── runtime.rs              # Script runtime
│   │   └── tests/
│   │
│   ├── blang_runtime/                  # Runtime support
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── allocator.rs            # Memory allocator
│   │   │   ├── panic.rs                # Panic handler
│   │   │   ├── signal.rs               # Reactive signals
│   │   │   ├── effect.rs               # Effect system
│   │   │   └── collections.rs          # Runtime collections
│   │   ├── wasm/                       # WASM-specific runtime
│   │   │   └── lib.rs
│   │   └── tests/
│   │
│   ├── blang_driver/                   # Compiler driver
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── driver.rs               # Main driver
│   │   │   ├── config.rs               # Compiler config
│   │   │   ├── session.rs              # Compilation session
│   │   │   └── queries.rs              # Query system
│   │   └── tests/
│   │
│   ├── blang_cli/                      # Command-line interface
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── build.rs            # Build command
│   │   │   │   ├── run.rs              # Run command
│   │   │   │   ├── check.rs            # Check command
│   │   │   │   └── init.rs             # Init command
│   │   │   ├── config.rs               # CLI config
│   │   │   └── ui.rs                   # User interface
│   │   └── tests/
│   │
│   └── blang_dev_server/               # Development server
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── server.rs               # HTTP server
│       │   ├── hmr.rs                  # Hot module replacement
│       │   ├── watch.rs                # File watching
│       │   └── ws.rs                   # WebSocket support
│       └── tests/
│
├── stdlib/                             # Standard library (Blang)
│   ├── core/                           # Core library
│   │   ├── lib.bl
│   │   ├── option.bl
│   │   └── result.bl
│   ├── collections/
│   │   ├── mod.bl
│   │   ├── vec.bl
│   │   └── map.bl
│   └── io/
│       └── mod.bl
│
├── runtime/                            # Runtime implementations
│   ├── wasm/
│   │   ├── allocator.wat               # WASM allocator
│   │   └── runtime.js                  # JS runtime glue
│   └── browser/
│       └── dom.js                      # DOM bindings
│
├── examples/                           # Example programs
│   ├── hello_world/
│   │   ├── main.bl
│   │   └── blang.toml
│   ├── fibonacci/
│   │   ├── main.bl
│   │   └── blang.toml
│   ├── counter_component/
│   │   ├── main.blang
│   │   └── blang.toml
│   └── data_pipeline/
│       ├── main.bs
│       └── blang.toml
│
├── tests/                              # Integration tests
│   ├── compile-pass/                   # Should compile
│   │   ├── basic_function.bl
│   │   └── generic_types.bl
│   ├── compile-fail/                   # Should fail
│   │   ├── type_mismatch.bl
│   │   └── borrow_error.bl
│   ├── run-pass/                       # Should run successfully
│   │   ├── fibonacci.bl
│   │   └── sorting.bl
│   └── ui/                             # Error message tests
│       └── type_errors.bl
│
├── benches/                            # Benchmarks
│   ├── compiler/
│   │   └── parse_benchmark.rs
│   └── runtime/
│       └── signal_benchmark.rs
│
├── tools/                              # Development tools
│   ├── test-runner/
│   │   └── src/main.rs
│   └── snapshot/
│       └── src/lib.rs
│
└── docs/                               # Documentation
    ├── book/                           # mdBook documentation
    │   ├── book.toml
    │   └── src/
    │       ├── SUMMARY.md
    │       ├── introduction.md
    │       └── getting-started.md
    └── api/                            # Generated API docs
```

---

## Crate Organization

### Core Compiler Pipeline (11 crates)

1. **blang_common** - Shared utilities
2. **blang_span** - Source location tracking
3. **blang_diagnostics** - Error reporting
4. **blang_lexer** - Tokenization
5. **blang_parser** - Parsing
6. **blang_ast** - AST definitions
7. **blang_resolve** - Name resolution
8. **blang_types** - Type representation
9. **blang_typeck** - Type checking
10. **blang_ir** - All IR levels (HIR, MIR, LIR)
11. **blang_borrow** - Borrow checking

### Code Generation (3 crates)

12. **blang_optimize** - Optimization passes
13. **blang_codegen_wasm** - WASM code generation
14. **blang_codegen_js** - JS glue generation

### Mode-Specific (2 crates)

15. **blang_component** - Component mode compilation
16. **blang_script** - Script mode compilation

### Runtime & Tools (3 crates)

17. **blang_runtime** - Runtime support (allocator, signals, etc.)
18. **blang_driver** - Compiler driver and orchestration
19. **blang_cli** - Command-line interface
20. **blang_dev_server** - Development server with HMR

---

## Crate Responsibilities

### 1. blang_common

**Responsibility**: Shared utilities and common types used across all crates.

**Public API**:
```rust
// Symbol interning
pub struct Symbol(u32);
pub struct Interner { ... }

// Arena allocation
pub struct Arena<T> { ... }

// Index types
pub struct IndexVec<I, T> { ... }

// Fast collections
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxHashSet<T> = HashSet<T, FxBuildHasher>;
```

**Dependencies**: None (pure utilities)

**Module Structure**:
- `common::arena` - Arena allocator
- `common::symbol` - String interning
- `common::index` - Newtype indices
- `common::collections` - Hash collections

---

### 2. blang_span

**Responsibility**: Track source code locations and manage source files.

**Public API**:
```rust
pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
}

pub struct SourceMap {
    files: Vec<SourceFile>,
}

pub struct SourceFile {
    pub name: FileName,
    pub src: String,
    pub lines: Vec<BytePos>,
}

pub struct BytePos(u32);
```

**Dependencies**:
- `blang_common` (Symbol for file names)

**Module Structure**:
- `span::span` - Span type
- `span::source_map` - Source file management
- `span::pos` - Position types

---

### 3. blang_diagnostics

**Responsibility**: Error and warning reporting with beautiful output.

**Public API**:
```rust
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub code: Option<DiagnosticCode>,
    pub labels: Vec<Label>,
}

pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

pub struct DiagnosticEmitter {
    source_map: Arc<SourceMap>,
}

impl DiagnosticEmitter {
    pub fn emit(&mut self, diag: &Diagnostic);
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (source locations)

**Module Structure**:
- `diagnostics::diagnostic` - Diagnostic types
- `diagnostics::emitter` - Output formatting
- `diagnostics::snippet` - Code snippet rendering
- `diagnostics::codes` - Error code definitions

---

### 4. blang_lexer

**Responsibility**: Convert source text into tokens.

**Public API**:
```rust
pub struct Lexer<'a> {
    source: &'a str,
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn next_token(&mut self) -> Token;
}

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub enum TokenKind {
    // Literals
    Integer { value: u128, suffix: Option<IntSuffix> },
    Float { value: f64, suffix: Option<FloatSuffix> },
    String { value: Symbol },
    // Keywords
    Fn, Let, If, Else, // ...
    // Operators
    Plus, Minus, Star, Slash, // ...
    // Delimiters
    OpenParen, CloseParen, // ...
}
```

**Dependencies**:
- `blang_common` (Symbol)
- `blang_span` (Span)

**Module Structure**:
- `lexer::lexer` - Main lexer
- `lexer::token` - Token types
- `lexer::cursor` - Character iterator
- `lexer::unescape` - String/char unescaping

---

### 5. blang_ast

**Responsibility**: Abstract syntax tree node definitions.

**Public API**:
```rust
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

pub enum ExprKind {
    Lit(Lit),
    Path(Path),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Block(Block),
    If(Box<Expr>, Block, Option<Box<Expr>>),
    // ... more variants
}

pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

pub struct Item {
    pub id: NodeId,
    pub kind: ItemKind,
    pub span: Span,
}

// Visitor pattern
pub trait Visitor<'ast> {
    fn visit_expr(&mut self, expr: &'ast Expr);
    fn visit_stmt(&mut self, stmt: &'ast Stmt);
    // ...
}
```

**Dependencies**:
- `blang_common` (Symbol, Arena)
- `blang_span` (Span)

**Module Structure**:
- `ast::node` - Base node types
- `ast::expr` - Expression nodes
- `ast::stmt` - Statement nodes
- `ast::item` - Item nodes
- `ast::ty` - Type nodes
- `ast::pat` - Pattern nodes
- `ast::attr` - Attribute nodes
- `ast::visit` - Visitor trait
- `ast::mut_visit` - Mutable visitor
- `ast::display` - Pretty printing

---

### 6. blang_parser

**Responsibility**: Parse token stream into AST.

**Public API**:
```rust
pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    diagnostics: DiagnosticEmitter,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], source_map: Arc<SourceMap>) -> Self;

    pub fn parse_file(&mut self) -> Result<SourceFile, Vec<Diagnostic>>;
    pub fn parse_expr(&mut self) -> Result<Expr, Vec<Diagnostic>>;
    pub fn parse_stmt(&mut self) -> Result<Stmt, Vec<Diagnostic>>;
    pub fn parse_item(&mut self) -> Result<Item, Vec<Diagnostic>>;
}

pub struct SourceFile {
    pub items: Vec<Item>,
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (Span)
- `blang_diagnostics` (error reporting)
- `blang_lexer` (Token)
- `blang_ast` (AST nodes)

**Module Structure**:
- `parser::parser` - Main parser
- `parser::expr` - Expression parsing
- `parser::stmt` - Statement parsing
- `parser::item` - Item parsing
- `parser::ty` - Type parsing
- `parser::pat` - Pattern parsing
- `parser::recovery` - Error recovery

---

### 7. blang_resolve

**Responsibility**: Resolve names to definitions and build scope trees.

**Public API**:
```rust
pub struct Resolver<'a> {
    scopes: ScopeTree,
    definitions: Definitions,
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self;
    pub fn resolve(&mut self, ast: &SourceFile) -> Result<ResolutionMap, Vec<Diagnostic>>;
}

pub struct ResolutionMap {
    // Maps NodeId to Definition
    resolutions: FxHashMap<NodeId, DefId>,
}

pub struct DefId(u32);

pub enum DefKind {
    Function,
    Variable,
    Type,
    Module,
}
```

**Dependencies**:
- `blang_common` (collections)
- `blang_span` (Span)
- `blang_diagnostics` (errors)
- `blang_ast` (AST)

**Module Structure**:
- `resolve::resolve` - Main resolver
- `resolve::scope` - Scope management
- `resolve::import` - Import resolution
- `resolve::def` - Definition tracking
- `resolve::path` - Path resolution

---

### 8. blang_types

**Responsibility**: Type representation and manipulation.

**Public API**:
```rust
pub enum Ty {
    Int(IntTy),
    Float(FloatTy),
    Bool,
    Str,
    Tuple(Vec<Ty>),
    Array(Box<Ty>, usize),
    Ref(Mutability, Box<Ty>),
    Fn(FnSig),
    Adt(AdtId, Vec<Ty>), // Algebraic data type (struct/enum)
    Param(ParamId), // Generic parameter
    Infer(InferVar), // Type variable for inference
}

pub struct FnSig {
    pub params: Vec<Ty>,
    pub ret: Box<Ty>,
}

pub struct TraitDef {
    pub name: Symbol,
    pub items: Vec<TraitItem>,
}
```

**Dependencies**:
- `blang_common` (Symbol, collections)
- `blang_span` (Span)

**Module Structure**:
- `types::ty` - Type representation
- `types::subst` - Type substitution
- `types::traits` - Trait definitions
- `types::layout` - Memory layout calculation

---

### 9. blang_typeck

**Responsibility**: Type checking and type inference.

**Public API**:
```rust
pub struct TypeChecker<'tcx> {
    types: &'tcx TypeContext,
    infer: InferContext<'tcx>,
}

impl<'tcx> TypeChecker<'tcx> {
    pub fn new(types: &'tcx TypeContext) -> Self;
    pub fn check_crate(&mut self, ast: &SourceFile) -> Result<TypedHir, Vec<Diagnostic>>;
}

pub struct InferContext<'tcx> {
    // Type inference context
    vars: Vec<InferVar>,
    constraints: Vec<Constraint>,
}

impl<'tcx> InferContext<'tcx> {
    pub fn fresh_var(&mut self) -> Ty;
    pub fn unify(&mut self, t1: Ty, t2: Ty) -> Result<(), TypeError>;
    pub fn solve(&mut self) -> Result<Substitution, Vec<TypeError>>;
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (Span)
- `blang_diagnostics` (errors)
- `blang_ast` (AST)
- `blang_types` (Ty)
- `blang_resolve` (ResolutionMap)

**Module Structure**:
- `typeck::check` - Type checking
- `typeck::infer` - Type inference
- `typeck::unify` - Unification algorithm
- `typeck::method` - Method resolution
- `typeck::coherence` - Trait coherence checking

---

### 10. blang_ir

**Responsibility**: All intermediate representations (HIR, MIR, LIR).

**Public API**:
```rust
// HIR - High-level IR (desugared AST)
pub mod hir {
    pub struct Expr { ... }
    pub struct Stmt { ... }
    pub fn lower_ast(ast: &ast::SourceFile) -> hir::Crate;
}

// MIR - Mid-level IR (control flow graph, SSA)
pub mod mir {
    pub struct Body {
        pub basic_blocks: Vec<BasicBlock>,
        pub locals: Vec<Local>,
    }

    pub struct BasicBlock {
        pub statements: Vec<Statement>,
        pub terminator: Terminator,
    }

    pub fn build_mir(hir: &hir::Crate) -> mir::Crate;
}

// LIR - Low-level IR (close to WASM)
pub mod lir {
    pub struct Instruction { ... }
    pub fn lower_mir(mir: &mir::Crate) -> lir::Module;
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (Span)
- `blang_ast` (for HIR lowering)
- `blang_types` (Ty)

**Module Structure**:
- `ir::hir` - High-level IR
  - `ir::hir::lower` - AST to HIR
  - `ir::hir::node` - HIR nodes
  - `ir::hir::visit` - Visitor
- `ir::mir` - Mid-level IR
  - `ir::mir::build` - HIR to MIR
  - `ir::mir::basic_block` - Basic blocks
  - `ir::mir::visit` - Visitor
- `ir::lir` - Low-level IR
  - `ir::lir::lower` - MIR to LIR
  - `ir::lir::inst` - Instructions

---

### 11. blang_borrow

**Responsibility**: Borrow checking and lifetime analysis.

**Public API**:
```rust
pub struct BorrowChecker<'mir> {
    mir: &'mir mir::Body,
    regions: RegionInference,
}

impl<'mir> BorrowChecker<'mir> {
    pub fn new(mir: &'mir mir::Body) -> Self;
    pub fn check(&mut self) -> Result<(), Vec<BorrowError>>;
}

pub struct BorrowError {
    pub kind: BorrowErrorKind,
    pub span: Span,
}

pub enum BorrowErrorKind {
    MutableBorrowWhileBorrowed,
    UseAfterMove,
    UseAfterFree,
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (Span)
- `blang_diagnostics` (errors)
- `blang_ir` (MIR)
- `blang_types` (Ty)

**Module Structure**:
- `borrow::check` - Borrow checker
- `borrow::lifetime` - Lifetime inference
- `borrow::region` - Region analysis

---

### 12. blang_optimize

**Responsibility**: Optimization passes on MIR.

**Public API**:
```rust
pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> Self;
    pub fn add_pass(&mut self, pass: Box<dyn Pass>);
    pub fn run(&mut self, mir: &mut mir::Crate);
}

pub trait Pass {
    fn name(&self) -> &str;
    fn run(&mut self, mir: &mut mir::Crate);
}

// Built-in passes
pub struct InlinePass { ... }
pub struct ConstPropPass { ... }
pub struct DeadCodeEliminationPass { ... }
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_ir` (MIR)
- `blang_types` (Ty)

**Module Structure**:
- `optimize::pass_manager` - Pass orchestration
- `optimize::inline` - Function inlining
- `optimize::const_prop` - Constant propagation
- `optimize::dce` - Dead code elimination
- `optimize::simplify_cfg` - CFG simplification

---

### 13. blang_codegen_wasm

**Responsibility**: Generate WebAssembly from LIR.

**Public API**:
```rust
pub struct WasmCodegen {
    module: wasm_encoder::Module,
}

impl WasmCodegen {
    pub fn new() -> Self;
    pub fn codegen(&mut self, lir: &lir::Module) -> Vec<u8>;
}

pub struct FunctionCodegen<'a> {
    func: wasm_encoder::Function,
}

impl<'a> FunctionCodegen<'a> {
    pub fn codegen_function(&mut self, func: &lir::Function);
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_ir` (LIR)
- `blang_types` (Ty for ABI)
- `wasm-encoder` (external crate)

**Module Structure**:
- `codegen_wasm::codegen` - Main codegen
- `codegen_wasm::module` - WASM module builder
- `codegen_wasm::function` - Function codegen
- `codegen_wasm::expr` - Expression codegen
- `codegen_wasm::memory` - Memory operations
- `codegen_wasm::abi` - Calling convention

---

### 14. blang_codegen_js

**Responsibility**: Generate JavaScript glue code.

**Public API**:
```rust
pub struct JsCodegen {
    output: String,
}

impl JsCodegen {
    pub fn new() -> Self;
    pub fn codegen(&mut self, lir: &lir::Module) -> String;
    pub fn codegen_bindings(&mut self, exports: &[Export]);
    pub fn codegen_dom_bindings(&mut self);
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_ir` (LIR)
- `blang_types` (Ty)

**Module Structure**:
- `codegen_js::codegen` - Main codegen
- `codegen_js::bindings` - WASM bindings
- `codegen_js::dom` - DOM API bindings
- `codegen_js::module` - ES module generation

---

### 15. blang_component

**Responsibility**: Compile component mode (.blang files).

**Public API**:
```rust
pub struct ComponentCompiler {
    parser: ComponentParser,
}

impl ComponentCompiler {
    pub fn new() -> Self;
    pub fn compile(&mut self, source: &str) -> Result<Component, Vec<Diagnostic>>;
}

pub struct Component {
    pub state: Vec<StateDecl>,
    pub props: Vec<PropDecl>,
    pub view: ViewTree,
    pub style: StyleSheet,
    pub methods: Vec<Method>,
}

pub struct ViewTree {
    pub root: ViewNode,
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (Span)
- `blang_diagnostics` (errors)
- `blang_parser` (for logic parsing)
- `blang_ast` (AST)

**Module Structure**:
- `component::parse` - Component parsing
- `component::view` - View tree compilation
- `component::style` - CSS compilation
- `component::reactive` - Reactivity codegen
- `component::lifecycle` - Lifecycle hooks

---

### 16. blang_script

**Responsibility**: Compile script mode (.bs files).

**Public API**:
```rust
pub struct ScriptCompiler {
    parser: ScriptParser,
}

impl ScriptCompiler {
    pub fn new() -> Self;
    pub fn compile(&mut self, source: &str) -> Result<Script, Vec<Diagnostic>>;
}

pub struct Script {
    pub jobs: Vec<Job>,
    pub config: ScriptConfig,
}

pub struct Job {
    pub name: Symbol,
    pub steps: Vec<Step>,
    pub dependencies: Vec<Symbol>,
}
```

**Dependencies**:
- `blang_common` (utilities)
- `blang_span` (Span)
- `blang_diagnostics` (errors)
- `blang_parser` (for logic parsing)
- `blang_ast` (AST)

**Module Structure**:
- `script::parse` - Script parsing
- `script::job` - Job compilation
- `script::schedule` - Dependency resolution
- `script::runtime` - Script runtime generation

---

### 17. blang_runtime

**Responsibility**: Runtime support code (allocator, signals, etc.).

**Public API**:
```rust
// Memory allocator
pub struct WasmAllocator { ... }

// Reactive signals
pub struct Signal<T> {
    value: Cell<T>,
    subscribers: Vec<EffectId>,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self;
    pub fn get(&self) -> T;
    pub fn set(&self, value: T);
    pub fn update(&self, f: impl FnOnce(T) -> T);
}

// Effect system
pub struct Effect {
    callback: Box<dyn Fn()>,
}

pub fn create_effect(f: impl Fn() + 'static);
```

**Dependencies**:
- `blang_common` (utilities)

**Module Structure**:
- `runtime::allocator` - Memory allocator
- `runtime::panic` - Panic handler
- `runtime::signal` - Signal implementation
- `runtime::effect` - Effect system
- `runtime::collections` - Runtime collections

---

### 18. blang_driver

**Responsibility**: Orchestrate the entire compilation pipeline.

**Public API**:
```rust
pub struct Driver {
    config: CompilerConfig,
    session: Session,
}

impl Driver {
    pub fn new(config: CompilerConfig) -> Self;
    pub fn compile(&mut self, input: Input) -> Result<Output, Vec<Diagnostic>>;
}

pub struct CompilerConfig {
    pub opt_level: OptLevel,
    pub target: Target,
    pub output_dir: PathBuf,
}

pub enum Input {
    File(PathBuf),
    Source(String),
}

pub struct Output {
    pub wasm: Vec<u8>,
    pub js: String,
}
```

**Dependencies**:
- All compiler crates
- `blang_common` (utilities)
- `blang_span` (SourceMap)
- `blang_diagnostics` (errors)

**Module Structure**:
- `driver::driver` - Main driver
- `driver::config` - Configuration
- `driver::session` - Compilation session
- `driver::queries` - Query-based compilation (future)

---

### 19. blang_cli

**Responsibility**: Command-line interface for the compiler.

**Public API** (binary, not library):
```rust
// Commands:
// blang build <path>
// blang run <path>
// blang check <path>
// blang init <name>
// blang dev
```

**Dependencies**:
- `blang_driver` (compiler)
- `blang_dev_server` (dev server)
- `clap` (CLI parsing)
- `colored` (terminal colors)

**Module Structure**:
- `cli::commands::build` - Build command
- `cli::commands::run` - Run command
- `cli::commands::check` - Type check only
- `cli::commands::init` - Project initialization
- `cli::config` - CLI configuration
- `cli::ui` - User interface utilities

---

### 20. blang_dev_server

**Responsibility**: Development server with HMR.

**Public API**:
```rust
pub struct DevServer {
    config: DevServerConfig,
    watcher: FileWatcher,
}

impl DevServer {
    pub fn new(config: DevServerConfig) -> Self;
    pub async fn start(&mut self) -> Result<(), Error>;
}

pub struct DevServerConfig {
    pub port: u16,
    pub host: String,
    pub root: PathBuf,
}
```

**Dependencies**:
- `blang_driver` (compiler)
- `tokio` (async runtime)
- `axum` or `warp` (HTTP server)
- `notify` (file watching)
- `tokio-tungstenite` (WebSocket)

**Module Structure**:
- `dev_server::server` - HTTP server
- `dev_server::hmr` - Hot module replacement
- `dev_server::watch` - File watching
- `dev_server::ws` - WebSocket support

---

## Dependency Graph

```
Level 0 (No dependencies):
  - blang_common

Level 1 (Depends only on common):
  - blang_span

Level 2:
  - blang_diagnostics (common, span)
  - blang_lexer (common, span)
  - blang_ast (common, span)
  - blang_types (common, span)

Level 3:
  - blang_parser (common, span, diagnostics, lexer, ast)
  - blang_resolve (common, span, diagnostics, ast)

Level 4:
  - blang_typeck (common, span, diagnostics, ast, types, resolve)
  - blang_ir (common, span, ast, types)

Level 5:
  - blang_borrow (common, span, diagnostics, ir, types)
  - blang_optimize (common, ir, types)
  - blang_component (common, span, diagnostics, parser, ast)
  - blang_script (common, span, diagnostics, parser, ast)

Level 6:
  - blang_codegen_wasm (common, ir, types)
  - blang_codegen_js (common, ir, types)

Level 7:
  - blang_driver (all compiler crates)

Level 8:
  - blang_cli (driver, dev_server)
  - blang_dev_server (driver)

Standalone:
  - blang_runtime (common only)
```

**Visualization**:
```
blang_common
    ↓
blang_span
    ↓
blang_diagnostics    blang_types
    ↓                     ↓
blang_lexer         blang_ast
    ↓                     ↓
blang_parser ←───────────┘
    ↓
blang_resolve
    ↓
blang_typeck ←── blang_types
    ↓
blang_ir
    ↓
├─→ blang_borrow
├─→ blang_optimize
├─→ blang_component
└─→ blang_script
    ↓
├─→ blang_codegen_wasm
└─→ blang_codegen_js
    ↓
blang_driver
    ↓
├─→ blang_cli
└─→ blang_dev_server

blang_runtime (independent)
```

---

## Module Naming Conventions

### General Rules

1. **Crate names**: `blang_*` (lowercase with underscore)
2. **Module names**: Snake case (e.g., `source_map`, `const_prop`)
3. **Type names**: Pascal case (e.g., `Expr`, `TokenKind`)
4. **Function names**: Snake case (e.g., `parse_expr`, `emit_diagnostic`)
5. **Constant names**: Screaming snake case (e.g., `MAX_LOCALS`)

### Module Organization

Each crate follows this structure:

```rust
// lib.rs - Public API and re-exports
pub mod submodule;

pub use submodule::PublicType;

// Internal implementation
mod internal;
```

### Common Module Names

- `lib.rs` - Crate root, public API
- `error.rs` - Error types specific to this crate
- `visit.rs` - Visitor pattern implementations
- `display.rs` or `fmt.rs` - Display/Debug implementations
- `test.rs` or `tests/` - Unit tests

### Naming Patterns

**AST/IR Nodes**:
- Node types: `Expr`, `Stmt`, `Item`
- Node kinds: `ExprKind`, `StmtKind`, `ItemKind`
- Node IDs: `NodeId`, `DefId`, `HirId`

**Visitors**:
- Immutable: `Visitor`, `visit_expr()`
- Mutable: `MutVisitor`, `visit_expr_mut()`

**Contexts**:
- Type context: `TypeContext`, `TyCtxt`
- Inference context: `InferContext`, `InferCtxt`
- Codegen context: `CodegenContext`, `CodegenCtxt`

**Builders**:
- `ExprBuilder`, `MirBuilder`, `ModuleBuilder`

**Utility Modules**:
- `util` - General utilities
- `intern` - String/data interning
- `arena` - Arena allocation
- `index` - Index types

---

## API Design Principles

### 1. Explicit Error Handling

```rust
// Good: Explicit Result types
pub fn parse_file(&mut self) -> Result<SourceFile, Vec<Diagnostic>>;

// Bad: Panicking on error
pub fn parse_file(&mut self) -> SourceFile; // panics on error
```

### 2. Ownership and Borrowing

```rust
// Good: Clear ownership
pub fn lower_ast(ast: SourceFile) -> hir::Crate; // Takes ownership
pub fn check_types(&mut self, ast: &SourceFile); // Borrows

// Bad: Unnecessary cloning
pub fn process(&self) -> ProcessedData {
    self.data.clone() // expensive!
}
```

### 3. Builder Pattern for Complex Types

```rust
// Good: Builder for complex construction
let expr = ExprBuilder::new()
    .kind(ExprKind::Binary(op, lhs, rhs))
    .span(span)
    .build();

// Bad: Direct construction with many fields
let expr = Expr {
    id: NodeId::new(),
    kind: ExprKind::Binary(op, lhs, rhs),
    span: span,
    ty: None,
    // ... many more fields
};
```

### 4. Use of Type State Pattern

```rust
// Good: Type states prevent misuse
pub struct Parser<State> { ... }

impl Parser<Initialized> {
    pub fn parse(self) -> (Parser<Parsed>, Result<Ast, Error>) { ... }
}

impl Parser<Parsed> {
    pub fn ast(&self) -> &Ast { ... }
}
```

### 5. Contextual Information

```rust
// Good: Include span/location in errors
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub span: Span,
}

// Bad: Errors without location
pub enum TypeError {
    Mismatch(Ty, Ty), // Where did this happen?
}
```

---

## Testing Strategy

### Unit Tests

Each crate includes unit tests in `tests/` or inline:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integer() {
        let lexer = Lexer::new("42");
        let token = lexer.next_token();
        assert_eq!(token.kind, TokenKind::Integer { value: 42, suffix: None });
    }
}
```

### Integration Tests

Top-level `tests/` directory:

```
tests/
├── compile-pass/    # Should compile successfully
├── compile-fail/    # Should fail with specific error
├── run-pass/        # Should compile and run successfully
└── ui/              # Error message quality tests
```

### Test Organization

```rust
// In tests/ directory
mod parse_tests {
    #[test]
    fn parse_function() { ... }

    #[test]
    fn parse_struct() { ... }
}

mod type_tests {
    #[test]
    fn infer_simple_types() { ... }

    #[test]
    fn generic_functions() { ... }
}
```

### Snapshot Testing

For AST and IR visualization:

```rust
use insta::assert_snapshot;

#[test]
fn test_parse_expression() {
    let source = "1 + 2 * 3";
    let ast = parse_expr(source);
    assert_snapshot!(format!("{:#?}", ast));
}
```

---

## Summary

This architecture provides:

1. **Clear Separation**: Each crate has a single responsibility
2. **Minimal Coupling**: Dependencies flow in one direction
3. **Testability**: Each crate can be tested independently
4. **Extensibility**: Easy to add new features or modes
5. **Performance**: Incremental compilation and caching
6. **Maintainability**: Clear module structure and naming

The structure supports the entire compilation pipeline from source code to WebAssembly, with support for all three execution modes (component, script, module), and provides a foundation for future features like LSP, formatter, and more.

**Next Steps**:
1. Create workspace Cargo.toml
2. Create individual crate Cargo.toml files
3. Begin implementation with foundation crates (M0)

For implementation details, see:
- LANGUAGE_SPEC.md for language features
- GRAMMAR.md for syntax
- ROADMAP.md for development plan
