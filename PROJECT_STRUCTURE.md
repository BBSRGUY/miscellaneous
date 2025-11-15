# Blang Project Structure

**Version**: 0.1
**Last Updated**: 2025-01-15

---

## Table of Contents

1. [Overview](#overview)
2. [Repository Layout](#repository-layout)
3. [Crate Organization](#crate-organization)
4. [Module Breakdown](#module-breakdown)
5. [Data Flow](#data-flow)
6. [Build Configuration](#build-configuration)
7. [Testing Structure](#testing-structure)
8. [Documentation Structure](#documentation-structure)
9. [Development Workflow](#development-workflow)

---

## Overview

The Blang compiler is organized as a Rust workspace with multiple crates, each responsible for a specific phase of compilation or runtime support. This modular structure enables:

- **Parallel development**: Different teams can work on different crates
- **Clear boundaries**: Each crate has a well-defined responsibility
- **Incremental compilation**: Cargo can recompile only changed crates
- **Code reuse**: Shared functionality is in common crates
- **Testing isolation**: Each crate can be tested independently

---

## Repository Layout

```
blang/
├── Cargo.toml                 # Workspace root
├── Cargo.lock                 # Locked dependencies
├── README.md                  # Project overview
├── LANGUAGE_SPEC.md           # Language specification
├── GRAMMAR.md                 # Grammar specification
├── ROADMAP.md                 # Development roadmap
├── PROJECT_STRUCTURE.md       # This file
├── CONTRIBUTING.md            # Contribution guidelines
├── LICENSE                    # License file
├── .gitignore                 # Git ignore rules
├── .github/                   # GitHub configuration
│   ├── workflows/             # CI/CD workflows
│   │   ├── ci.yml             # Main CI pipeline
│   │   ├── release.yml        # Release workflow
│   │   └── docs.yml           # Documentation build
│   ├── ISSUE_TEMPLATE/        # Issue templates
│   └── PULL_REQUEST_TEMPLATE.md
├── docs/                      # Documentation
│   ├── book/                  # Language guide (mdBook)
│   ├── reference/             # Language reference
│   ├── api/                   # API documentation
│   └── examples/              # Example code
├── crates/                    # All Rust crates
│   ├── blang-common/          # Shared utilities
│   ├── blang-span/            # Source position tracking
│   ├── blang-diagnostics/     # Error reporting
│   ├── blang-syntax/          # Lexer and parser
│   ├── blang-ast/             # Abstract syntax tree
│   ├── blang-hir/             # High-level IR
│   ├── blang-mir/             # Mid-level IR
│   ├── blang-lir/             # Low-level IR
│   ├── blang-typeck/          # Type checker
│   ├── blang-resolve/         # Name resolution
│   ├── blang-borrow/          # Borrow checker
│   ├── blang-codegen-wasm/    # WASM code generator
│   ├── blang-codegen-js/      # JS glue generator
│   ├── blang-runtime/         # Runtime support
│   ├── blang-component/       # Component mode compiler
│   ├── blang-script/          # Script mode compiler
│   ├── blang-optimize/        # Optimization passes
│   ├── blang-cli/             # Command-line interface
│   ├── blang-lsp/             # Language server
│   ├── blang-fmt/             # Code formatter
│   └── blang-driver/          # Compiler driver
├── stdlib/                    # Standard library
│   ├── core/                  # Core library (no_std)
│   ├── std/                   # Standard library
│   ├── collections/           # Data structures
│   ├── io/                    # I/O operations
│   ├── http/                  # HTTP client
│   ├── json/                  # JSON serialization
│   └── test/                  # Testing framework
├── runtime/                   # Runtime implementations
│   ├── wasm/                  # WASM runtime
│   ├── browser/               # Browser-specific code
│   └── allocator/             # Memory allocator
├── examples/                  # Example applications
│   ├── todo-app/              # Todo application
│   ├── counter/               # Simple counter
│   ├── fibonacci/             # Fibonacci module
│   └── pipeline/              # Script example
├── tests/                     # Integration tests
│   ├── compile-pass/          # Should compile successfully
│   ├── compile-fail/          # Should fail to compile
│   ├── run-pass/              # Should run successfully
│   ├── run-fail/              # Should fail at runtime
│   └── ui/                    # UI tests (error messages)
├── benches/                   # Benchmarks
│   ├── compiler/              # Compiler benchmarks
│   └── runtime/               # Runtime benchmarks
└── tools/                     # Development tools
    ├── test-runner/           # Test harness
    ├── snapshot/              # Snapshot testing
    └── coverage/              # Coverage tools
```

---

## Crate Organization

### Workspace Configuration

**Cargo.toml** (workspace root):

```toml
[workspace]
members = [
    "crates/blang-common",
    "crates/blang-span",
    "crates/blang-diagnostics",
    "crates/blang-syntax",
    "crates/blang-ast",
    "crates/blang-hir",
    "crates/blang-mir",
    "crates/blang-lir",
    "crates/blang-typeck",
    "crates/blang-resolve",
    "crates/blang-borrow",
    "crates/blang-codegen-wasm",
    "crates/blang-codegen-js",
    "crates/blang-runtime",
    "crates/blang-component",
    "crates/blang-script",
    "crates/blang-optimize",
    "crates/blang-cli",
    "crates/blang-lsp",
    "crates/blang-fmt",
    "crates/blang-driver",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Blang Team"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/blang/blang"

[workspace.dependencies]
# Shared dependencies
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
indexmap = "2.0"
rustc-hash = "1.1"
parking_lot = "0.12"

# WASM tooling
wasmparser = "0.118"
wasm-encoder = "0.38"
walrus = "0.20"

# CLI tools
clap = { version = "4.4", features = ["derive"] }
colored = "2.0"

# LSP
tower-lsp = "0.20"
lsp-types = "0.95"

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true

[profile.test]
opt-level = 1

[profile.bench]
opt-level = 3
```

---

## Module Breakdown

### 1. blang-common

**Purpose**: Shared utilities and common types

**Structure**:
```
blang-common/
├── src/
│   ├── lib.rs
│   ├── arena.rs              # Arena allocator
│   ├── symbol.rs             # Symbol interning
│   ├── index.rs              # Index types
│   ├── collections.rs        # Common collections
│   └── util.rs               # Utility functions
├── Cargo.toml
└── tests/
```

**Key Types**:
- `Symbol`: Interned string identifier
- `Arena<T>`: Typed arena allocator
- `IndexVec<I, T>`: Vector indexed by newtype
- `FxHashMap`, `FxHashSet`: Fast hash collections

---

### 2. blang-span

**Purpose**: Source location tracking

**Structure**:
```
blang-span/
├── src/
│   ├── lib.rs
│   ├── span.rs               # Span type
│   ├── source_map.rs         # Source file tracking
│   ├── pos.rs                # Position types
│   └── hygiene.rs            # Macro hygiene
├── Cargo.toml
└── tests/
```

**Key Types**:
- `Span`: Represents a source code range
- `SourceMap`: Manages source files
- `BytePos`, `CharPos`: Position types
- `SourceFile`: Individual source file

---

### 3. blang-diagnostics

**Purpose**: Error and warning reporting

**Structure**:
```
blang-diagnostics/
├── src/
│   ├── lib.rs
│   ├── diagnostic.rs         # Diagnostic types
│   ├── emitter.rs            # Diagnostic output
│   ├── snippet.rs            # Code snippets
│   ├── style.rs              # Output styling
│   └── codes.rs              # Error codes
├── Cargo.toml
└── tests/
```

**Key Types**:
- `Diagnostic`: Error/warning representation
- `DiagnosticEmitter`: Outputs diagnostics
- `Severity`: Error, Warning, Note, Help
- `Label`: Annotates source locations

---

### 4. blang-syntax

**Purpose**: Lexical analysis and parsing

**Structure**:
```
blang-syntax/
├── src/
│   ├── lib.rs
│   ├── lexer/
│   │   ├── mod.rs            # Lexer main module
│   │   ├── token.rs          # Token types
│   │   ├── cursor.rs         # Character cursor
│   │   └── unescape.rs       # String unescaping
│   ├── parser/
│   │   ├── mod.rs            # Parser main module
│   │   ├── expr.rs           # Expression parsing
│   │   ├── stmt.rs           # Statement parsing
│   │   ├── item.rs           # Item parsing
│   │   ├── ty.rs             # Type parsing
│   │   ├── pat.rs            # Pattern parsing
│   │   └── recovery.rs       # Error recovery
│   ├── validate.rs           # AST validation
│   └── token_stream.rs       # Token stream
├── Cargo.toml
└── tests/
    ├── lexer/
    └── parser/
```

**Key Types**:
- `Token`: Lexical token
- `Lexer`: Tokenizer
- `Parser`: Recursive descent parser
- `TokenStream`: Stream of tokens

---

### 5. blang-ast

**Purpose**: Abstract syntax tree definitions

**Structure**:
```
blang-ast/
├── src/
│   ├── lib.rs
│   ├── ast.rs                # AST node definitions
│   ├── expr.rs               # Expression nodes
│   ├── stmt.rs               # Statement nodes
│   ├── item.rs               # Item nodes
│   ├── ty.rs                 # Type nodes
│   ├── pat.rs                # Pattern nodes
│   ├── attr.rs               # Attributes
│   ├── visit.rs              # Visitor trait
│   ├── mut_visit.rs          # Mutable visitor
│   └── pretty.rs             # Pretty printing
├── Cargo.toml
└── tests/
```

**Key Types**:
- `Expr`: Expression AST
- `Stmt`: Statement AST
- `Item`: Top-level item AST
- `Type`: Type expression AST
- `Pat`: Pattern AST

---

### 6. blang-resolve

**Purpose**: Name resolution and scope analysis

**Structure**:
```
blang-resolve/
├── src/
│   ├── lib.rs
│   ├── resolve.rs            # Main resolver
│   ├── scope.rs              # Scope management
│   ├── import.rs             # Import resolution
│   ├── def.rs                # Definition tracking
│   └── path.rs               # Path resolution
├── Cargo.toml
└── tests/
```

**Key Types**:
- `Resolver`: Name resolution engine
- `Scope`: Lexical scope
- `Def`: Definition information
- `Res`: Resolution result

---

### 7. blang-hir

**Purpose**: High-level intermediate representation

**Structure**:
```
blang-hir/
├── src/
│   ├── lib.rs
│   ├── hir.rs                # HIR node definitions
│   ├── lower.rs              # AST to HIR lowering
│   ├── expr.rs               # HIR expressions
│   ├── stmt.rs               # HIR statements
│   ├── item.rs               # HIR items
│   ├── ty.rs                 # HIR types
│   ├── pat.rs                # HIR patterns
│   ├── visit.rs              # Visitor trait
│   └── pretty.rs             # Pretty printing
├── Cargo.toml
└── tests/
```

**Key Types**:
- `HirExpr`: Desugared expression
- `HirStmt`: Desugared statement
- `HirItem`: Desugared item
- `HirId`: Unique HIR node ID

---

### 8. blang-typeck

**Purpose**: Type checking and inference

**Structure**:
```
blang-typeck/
├── src/
│   ├── lib.rs
│   ├── typeck.rs             # Type checker
│   ├── infer.rs              # Type inference
│   ├── unify.rs              # Unification
│   ├── subtype.rs            # Subtyping
│   ├── traits.rs             # Trait resolution
│   ├── method.rs             # Method resolution
│   ├── coherence.rs          # Trait coherence
│   └── ty.rs                 # Type representation
├── Cargo.toml
└── tests/
```

**Key Types**:
- `TypeChecker`: Type checking context
- `InferCtxt`: Inference context
- `Ty`: Type representation
- `TyVar`: Type variable

---

### 9. blang-mir

**Purpose**: Mid-level intermediate representation

**Structure**:
```
blang-mir/
├── src/
│   ├── lib.rs
│   ├── mir.rs                # MIR definitions
│   ├── build.rs              # HIR to MIR building
│   ├── cfg.rs                # Control flow graph
│   ├── ssa.rs                # SSA construction
│   ├── visit.rs              # Visitor trait
│   └── pretty.rs             # Pretty printing
├── Cargo.toml
└── tests/
```

**Key Types**:
- `Body`: MIR function body
- `BasicBlock`: Basic block
- `Statement`: MIR statement
- `Terminator`: Block terminator

---

### 10. blang-borrow

**Purpose**: Borrow checking and lifetime analysis

**Structure**:
```
blang-borrow/
├── src/
│   ├── lib.rs
│   ├── borrow.rs             # Borrow checker
│   ├── lifetime.rs           # Lifetime inference
│   ├── region.rs             # Region analysis
│   └── polonius.rs           # Polonius integration
├── Cargo.toml
└── tests/
```

**Key Types**:
- `BorrowChecker`: Borrow checking engine
- `Region`: Lifetime region
- `BorrowKind`: Shared/mutable/unique

---

### 11. blang-optimize

**Purpose**: Optimization passes

**Structure**:
```
blang-optimize/
├── src/
│   ├── lib.rs
│   ├── passes/
│   │   ├── mod.rs
│   │   ├── inline.rs         # Inline expansion
│   │   ├── const_prop.rs     # Constant propagation
│   │   ├── dce.rs            # Dead code elimination
│   │   ├── cse.rs            # Common subexpression
│   │   └── simplify.rs       # CFG simplification
│   └── pass_manager.rs       # Pass orchestration
├── Cargo.toml
└── tests/
```

---

### 12. blang-lir

**Purpose**: Low-level intermediate representation

**Structure**:
```
blang-lir/
├── src/
│   ├── lib.rs
│   ├── lir.rs                # LIR definitions
│   ├── lower.rs              # MIR to LIR lowering
│   └── pretty.rs             # Pretty printing
├── Cargo.toml
└── tests/
```

---

### 13. blang-codegen-wasm

**Purpose**: WebAssembly code generation

**Structure**:
```
blang-codegen-wasm/
├── src/
│   ├── lib.rs
│   ├── codegen.rs            # Main codegen
│   ├── module.rs             # WASM module builder
│   ├── function.rs           # Function codegen
│   ├── expr.rs               # Expression codegen
│   ├── memory.rs             # Memory management
│   ├── abi.rs                # Calling convention
│   └── intrinsics.rs         # Intrinsic functions
├── Cargo.toml
└── tests/
```

---

### 14. blang-codegen-js

**Purpose**: JavaScript glue code generation

**Structure**:
```
blang-codegen-js/
├── src/
│   ├── lib.rs
│   ├── codegen.rs            # JS codegen
│   ├── bindings.rs           # WASM bindings
│   ├── dom.rs                # DOM bindings
│   ├── interop.rs            # JS interop
│   └── module.rs             # ES module generation
├── Cargo.toml
└── tests/
```

---

### 15. blang-component

**Purpose**: Component mode compilation

**Structure**:
```
blang-component/
├── src/
│   ├── lib.rs
│   ├── parser.rs             # Component parsing
│   ├── view.rs               # View compilation
│   ├── style.rs              # Style compilation
│   ├── reactive.rs           # Reactivity codegen
│   └── lifecycle.rs          # Lifecycle hooks
├── Cargo.toml
└── tests/
```

---

### 16. blang-script

**Purpose**: Script mode compilation

**Structure**:
```
blang-script/
├── src/
│   ├── lib.rs
│   ├── parser.rs             # Script parsing
│   ├── job.rs                # Job compilation
│   ├── step.rs               # Step compilation
│   ├── schedule.rs           # Scheduling
│   └── runtime.rs            # Script runtime
├── Cargo.toml
└── tests/
```

---

### 17. blang-runtime

**Purpose**: Runtime support code

**Structure**:
```
blang-runtime/
├── src/
│   ├── lib.rs
│   ├── allocator.rs          # Memory allocator
│   ├── panic.rs              # Panic handler
│   ├── signal.rs             # Reactive signals
│   ├── effect.rs             # Effect system
│   ├── actor.rs              # Actor runtime
│   └── channel.rs            # Channel implementation
├── Cargo.toml
└── tests/
```

---

### 18. blang-driver

**Purpose**: Compiler driver and orchestration

**Structure**:
```
blang-driver/
├── src/
│   ├── lib.rs
│   ├── driver.rs             # Main driver
│   ├── config.rs             # Compiler config
│   ├── session.rs            # Compilation session
│   ├── queries.rs            # Query system
│   └── incremental.rs        # Incremental compilation
├── Cargo.toml
└── tests/
```

---

### 19. blang-cli

**Purpose**: Command-line interface

**Structure**:
```
blang-cli/
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── build.rs          # Build command
│   │   ├── run.rs            # Run command
│   │   ├── test.rs           # Test command
│   │   ├── fmt.rs            # Format command
│   │   ├── check.rs          # Check command
│   │   └── init.rs           # Init command
│   ├── config.rs             # CLI configuration
│   └── ui.rs                 # User interface
├── Cargo.toml
└── tests/
```

---

### 20. blang-lsp

**Purpose**: Language Server Protocol implementation

**Structure**:
```
blang-lsp/
├── src/
│   ├── main.rs
│   ├── server.rs             # LSP server
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── completion.rs     # Autocomplete
│   │   ├── hover.rs          # Hover info
│   │   ├── definition.rs     # Go to definition
│   │   ├── references.rs     # Find references
│   │   └── diagnostics.rs    # Diagnostics
│   └── state.rs              # Server state
├── Cargo.toml
└── tests/
```

---

### 21. blang-fmt

**Purpose**: Code formatter

**Structure**:
```
blang-fmt/
├── src/
│   ├── lib.rs
│   ├── format.rs             # Formatting engine
│   ├── config.rs             # Format configuration
│   ├── expr.rs               # Expression formatting
│   ├── stmt.rs               # Statement formatting
│   └── item.rs               # Item formatting
├── Cargo.toml
└── tests/
```

---

## Data Flow

### Compilation Pipeline

```
Source Code (.blang, .bl, .bs)
    ↓
[blang-syntax] Lexer
    ↓
Tokens
    ↓
[blang-syntax] Parser
    ↓
AST (blang-ast)
    ↓
[blang-resolve] Name Resolution
    ↓
Resolved AST
    ↓
[blang-hir] HIR Lowering
    ↓
HIR (blang-hir)
    ↓
[blang-typeck] Type Checking
    ↓
Typed HIR
    ↓
[blang-mir] MIR Building
    ↓
MIR (blang-mir)
    ↓
[blang-borrow] Borrow Checking
    ↓
Validated MIR
    ↓
[blang-optimize] Optimization Passes
    ↓
Optimized MIR
    ↓
[blang-lir] LIR Lowering
    ↓
LIR (blang-lir)
    ↓
[blang-codegen-wasm] WASM Codegen
    ↓
WASM Binary
    ↓
[blang-codegen-js] JS Glue Generation
    ↓
JavaScript Glue Code
    ↓
Output Bundle
```

---

## Build Configuration

### Feature Flags

```toml
[features]
default = ["std"]
std = []
no_std = []
incremental = ["blang-driver/incremental"]
lsp = ["blang-lsp"]
fmt = ["blang-fmt"]
```

### Platform Support

- **Primary**: wasm32-unknown-unknown
- **Development**: x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc
- **Future**: wasm32-wasi, aarch64-apple-darwin

---

## Testing Structure

### Unit Tests

Each crate has `tests/` subdirectory:

```rust
// In crates/blang-syntax/src/lexer/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_integer() {
        let tokens = Lexer::new("42").collect::<Vec<_>>();
        assert_eq!(tokens[0].kind, TokenKind::Integer);
    }
}
```

### Integration Tests

In workspace `tests/` directory:

```
tests/
├── compile-pass/
│   ├── hello.bl
│   └── fibonacci.bl
├── compile-fail/
│   ├── type-error.bl
│   └── syntax-error.bl
├── run-pass/
│   ├── simple.bl
│   └── complex.bl
└── ui/
    ├── error-messages.bl
    └── warnings.bl
```

### Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn test_parse_function() {
    let source = "fn foo() -> i32 { 42 }";
    let ast = parse(source);
    assert_snapshot!(format!("{:#?}", ast));
}
```

---

## Documentation Structure

### Code Documentation

```rust
/// Parses a source file into an AST.
///
/// # Arguments
///
/// * `source` - The source code to parse
/// * `file_id` - The ID of the source file
///
/// # Returns
///
/// The parsed AST on success, or parse errors.
///
/// # Examples
///
/// ```
/// let ast = parse("fn main() {}", FileId(0))?;
/// ```
pub fn parse(source: &str, file_id: FileId) -> Result<Ast, Vec<ParseError>> {
    // ...
}
```

### Book Structure

```
docs/book/
├── src/
│   ├── SUMMARY.md
│   ├── introduction.md
│   ├── getting-started/
│   ├── language-guide/
│   ├── component-mode/
│   ├── script-mode/
│   ├── module-mode/
│   ├── advanced/
│   └── reference/
└── book.toml
```

---

## Development Workflow

### Getting Started

```bash
# Clone repository
git clone https://github.com/blang/blang.git
cd blang

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p blang-syntax

# Check code
cargo clippy --workspace

# Format code
cargo fmt --workspace
```

### Adding a New Feature

1. Update language spec in `LANGUAGE_SPEC.md`
2. Update grammar in `GRAMMAR.md`
3. Add AST nodes in `blang-ast`
4. Update parser in `blang-syntax`
5. Add type checking in `blang-typeck`
6. Add codegen in `blang-codegen-wasm`
7. Write tests
8. Update documentation

### Running Examples

```bash
# Build example
cargo run --bin blang -- build examples/counter/

# Run example
cargo run --bin blang -- run examples/counter/

# Format example
cargo run --bin blang -- fmt examples/counter/
```

---

## Continuous Integration

### CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --workspace -- --check

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/tarpaulin@v0.1
      - uses: codecov/codecov-action@v3
```

---

## Conclusion

This project structure provides a solid foundation for building the Blang compiler. The modular design allows for:

- **Clear separation of concerns**
- **Independent testing and development**
- **Incremental compilation**
- **Easy onboarding for contributors**

As the project evolves, this structure may be refined, but the core principles should remain:

1. **Modularity**: Small, focused crates
2. **Testability**: Comprehensive test coverage
3. **Documentation**: Clear, helpful documentation
4. **Maintainability**: Clean, idiomatic Rust code

For detailed language features, see LANGUAGE_SPEC.md.
For grammar details, see GRAMMAR.md.
For development timeline, see ROADMAP.md.
