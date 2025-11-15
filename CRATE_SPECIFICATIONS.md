# Blang Crate Specifications

**Version**: 0.1
**Date**: 2025-01-15

This document contains the complete Cargo.toml configuration for each crate in the Blang workspace.

---

## Table of Contents

1. [Foundation Crates](#foundation-crates)
2. [Frontend Crates](#frontend-crates)
3. [Middle-end Crates](#middle-end-crates)
4. [Backend Crates](#backend-crates)
5. [Mode-Specific Crates](#mode-specific-crates)
6. [Runtime & Tools](#runtime--tools)

---

## Foundation Crates

### 1. blang_common

**Path**: `crates/blang_common/Cargo.toml`

```toml
[package]
name = "blang_common"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Hash functions
rustc-hash.workspace = true

# Utilities
once_cell.workspace = true
parking_lot.workspace = true

# Serialization (optional, for debugging)
serde = { workspace = true, optional = true }

[features]
default = []
serde = ["dep:serde"]

[dev-dependencies]
criterion.workspace = true
```

**Purpose**: Shared utilities (symbol interning, arena allocation, fast collections)

**No dependencies on other Blang crates**

---

### 2. blang_span

**Path**: `crates/blang_span/Cargo.toml`

```toml
[package]
name = "blang_span"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true

# Utilities
parking_lot.workspace = true

# Serialization
serde = { workspace = true, optional = true }

[features]
default = []
serde = ["dep:serde", "blang_common/serde"]

[dev-dependencies]
```

**Purpose**: Source location tracking and source file management

**Dependencies**:
- `blang_common` for Symbol

---

### 3. blang_diagnostics

**Path**: `crates/blang_diagnostics/Cargo.toml`

```toml
[package]
name = "blang_diagnostics"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true

# Terminal output
colored.workspace = true

# Unicode handling
unicode-width = "0.1"

# Error handling
thiserror.workspace = true

[dev-dependencies]
```

**Purpose**: Error and warning reporting with beautiful terminal output

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for source locations

---

## Frontend Crates

### 4. blang_lexer

**Path**: `crates/blang_lexer/Cargo.toml`

```toml
[package]
name = "blang_lexer"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true

# Unicode handling
unicode-xid = "0.2"

# Error handling
thiserror.workspace = true

[dev-dependencies]
insta.workspace = true
proptest.workspace = true

[lib]
doctest = false
```

**Purpose**: Tokenization of Blang source code

**Dependencies**:
- `blang_common` for Symbol
- `blang_span` for Span

---

### 5. blang_ast

**Path**: `crates/blang_ast/Cargo.toml`

```toml
[package]
name = "blang_ast"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true

# Data structures
smallvec.workspace = true

# Serialization
serde = { workspace = true, optional = true }

[features]
default = []
serde = ["dep:serde", "blang_common/serde", "blang_span/serde"]

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Abstract syntax tree node definitions

**Dependencies**:
- `blang_common` for Symbol, Arena
- `blang_span` for Span

---

### 6. blang_parser

**Path**: `crates/blang_parser/Cargo.toml`

```toml
[package]
name = "blang_parser"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_lexer.workspace = true
blang_ast.workspace = true

# Error handling
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
proptest.workspace = true
tracing-subscriber.workspace = true

[lib]
doctest = false
```

**Purpose**: Parse tokens into AST

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for Span
- `blang_diagnostics` for error reporting
- `blang_lexer` for Token
- `blang_ast` for AST nodes

---

## Middle-end Crates

### 7. blang_resolve

**Path**: `crates/blang_resolve/Cargo.toml`

```toml
[package]
name = "blang_resolve"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_ast.workspace = true

# Data structures
indexmap.workspace = true
smallvec.workspace = true

# Error handling
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Name resolution and scope analysis

**Dependencies**:
- `blang_common` for collections
- `blang_span` for Span
- `blang_diagnostics` for errors
- `blang_ast` for AST

---

### 8. blang_types

**Path**: `crates/blang_types/Cargo.toml`

```toml
[package]
name = "blang_types"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true

# Data structures
smallvec.workspace = true
indexmap.workspace = true

# Utilities
bitflags.workspace = true

# Serialization
serde = { workspace = true, optional = true }

[features]
default = []
serde = ["dep:serde", "blang_common/serde"]

[dev-dependencies]
```

**Purpose**: Type representation and manipulation

**Dependencies**:
- `blang_common` for Symbol, collections
- `blang_span` for Span

---

### 9. blang_typeck

**Path**: `crates/blang_typeck/Cargo.toml`

```toml
[package]
name = "blang_typeck"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_ast.workspace = true
blang_resolve.workspace = true
blang_types.workspace = true

# Data structures
indexmap.workspace = true
smallvec.workspace = true

# Error handling
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Type checking and type inference

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for Span
- `blang_diagnostics` for errors
- `blang_ast` for AST
- `blang_resolve` for name resolution
- `blang_types` for Ty

---

### 10. blang_ir

**Path**: `crates/blang_ir/Cargo.toml`

```toml
[package]
name = "blang_ir"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_ast.workspace = true
blang_types.workspace = true

# Data structures
smallvec.workspace = true
indexmap.workspace = true

# Serialization
serde = { workspace = true, optional = true }

# Logging
tracing.workspace = true

[features]
default = []
serde = ["dep:serde", "blang_common/serde", "blang_types/serde"]

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Intermediate representations (HIR, MIR, LIR)

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for Span
- `blang_ast` for HIR lowering
- `blang_types` for Ty

---

### 11. blang_borrow

**Path**: `crates/blang_borrow/Cargo.toml`

```toml
[package]
name = "blang_borrow"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_ir.workspace = true
blang_types.workspace = true

# Data structures
indexmap.workspace = true
smallvec.workspace = true

# Error handling
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Borrow checking and lifetime analysis

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for Span
- `blang_diagnostics` for errors
- `blang_ir` for MIR
- `blang_types` for Ty

---

### 12. blang_optimize

**Path**: `crates/blang_optimize/Cargo.toml`

```toml
[package]
name = "blang_optimize"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_ir.workspace = true
blang_types.workspace = true

# Data structures
indexmap.workspace = true
smallvec.workspace = true
bitflags.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Optimization passes on MIR

**Dependencies**:
- `blang_common` for utilities
- `blang_ir` for MIR
- `blang_types` for Ty

---

## Backend Crates

### 13. blang_codegen_wasm

**Path**: `crates/blang_codegen_wasm/Cargo.toml`

```toml
[package]
name = "blang_codegen_wasm"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_ir.workspace = true
blang_types.workspace = true

# WebAssembly
wasm-encoder.workspace = true
wasmparser.workspace = true

# Data structures
indexmap.workspace = true
smallvec.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
walrus.workspace = true
```

**Purpose**: Generate WebAssembly from LIR

**Dependencies**:
- `blang_common` for utilities
- `blang_ir` for LIR
- `blang_types` for Ty (ABI)

---

### 14. blang_codegen_js

**Path**: `crates/blang_codegen_js/Cargo.toml`

```toml
[package]
name = "blang_codegen_js"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_ir.workspace = true
blang_types.workspace = true

# Code generation
handlebars = "5.0"

# Data structures
indexmap.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
```

**Purpose**: Generate JavaScript glue code

**Dependencies**:
- `blang_common` for utilities
- `blang_ir` for LIR
- `blang_types` for Ty

---

## Mode-Specific Crates

### 15. blang_component

**Path**: `crates/blang_component/Cargo.toml`

```toml
[package]
name = "blang_component"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_lexer.workspace = true
blang_parser.workspace = true
blang_ast.workspace = true

# CSS parsing
lightningcss = "1.0.0-alpha.51"

# HTML parsing
html5ever = "0.26"

# Data structures
indexmap.workspace = true
smallvec.workspace = true

# Error handling
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Component mode (.blang files) compilation

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for Span
- `blang_diagnostics` for errors
- `blang_lexer` and `blang_parser` for parsing logic blocks
- `blang_ast` for AST

---

### 16. blang_script

**Path**: `crates/blang_script/Cargo.toml`

```toml
[package]
name = "blang_script"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_lexer.workspace = true
blang_parser.workspace = true
blang_ast.workspace = true

# Graph algorithms (for dependency resolution)
petgraph = "0.6"

# Data structures
indexmap.workspace = true
smallvec.workspace = true

# Error handling
thiserror.workspace = true

# Logging
tracing.workspace = true

[dev-dependencies]
insta.workspace = true
```

**Purpose**: Script mode (.bs files) compilation

**Dependencies**:
- `blang_common` for utilities
- `blang_span` for Span
- `blang_diagnostics` for errors
- `blang_lexer` and `blang_parser` for parsing
- `blang_ast` for AST

---

## Runtime & Tools

### 17. blang_runtime

**Path**: `crates/blang_runtime/Cargo.toml`

```toml
[package]
name = "blang_runtime"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal (minimal)
blang_common.workspace = true

# Async runtime (for async features)
futures = { version = "0.3", optional = true }

# Synchronization
parking_lot.workspace = true

# Collections
indexmap.workspace = true

[features]
default = ["std"]
std = []
no_std = []
async = ["futures"]

# WASM target
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["Window", "Document", "Element", "HtmlElement"] }

[dev-dependencies]
```

**Purpose**: Runtime support (allocator, signals, effects, collections)

**Dependencies**:
- `blang_common` for utilities
- Minimal external dependencies

---

### 18. blang_driver

**Path**: `crates/blang_driver/Cargo.toml`

```toml
[package]
name = "blang_driver"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal - all compiler crates
blang_common.workspace = true
blang_span.workspace = true
blang_diagnostics.workspace = true
blang_lexer.workspace = true
blang_ast.workspace = true
blang_parser.workspace = true
blang_resolve.workspace = true
blang_types.workspace = true
blang_typeck.workspace = true
blang_ir.workspace = true
blang_borrow.workspace = true
blang_optimize.workspace = true
blang_codegen_wasm.workspace = true
blang_codegen_js.workspace = true
blang_component.workspace = true
blang_script.workspace = true

# File system
walkdir = "2.4"

# Serialization (for config files)
serde.workspace = true
serde_json.workspace = true
toml = "0.8"

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Logging
tracing.workspace = true
tracing-subscriber.workspace = true

[dev-dependencies]
tempfile = "3.8"
```

**Purpose**: Orchestrate the entire compilation pipeline

**Dependencies**:
- All compiler crates
- `blang_common` for utilities
- `blang_span` for SourceMap
- `blang_diagnostics` for error reporting

---

### 19. blang_cli

**Path**: `crates/blang_cli/Cargo.toml`

```toml
[package]
name = "blang_cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[[bin]]
name = "blang"
path = "src/main.rs"

[dependencies]
# Internal
blang_driver.workspace = true
blang_diagnostics.workspace = true
blang_dev_server = { workspace = true, optional = true }

# CLI
clap.workspace = true
colored.workspace = true
indicatif.workspace = true

# File system
walkdir = "2.4"

# Configuration
toml = "0.8"
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true

# Logging
tracing.workspace = true
tracing-subscriber.workspace = true

[features]
default = ["dev-server"]
dev-server = ["dep:blang_dev_server"]

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.8"
```

**Purpose**: Command-line interface

**Dependencies**:
- `blang_driver` for compilation
- `blang_dev_server` (optional) for dev server

---

### 20. blang_dev_server

**Path**: `crates/blang_dev_server/Cargo.toml`

```toml
[package]
name = "blang_dev_server"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Internal
blang_driver.workspace = true
blang_diagnostics.workspace = true

# Async runtime
tokio.workspace = true

# HTTP server
axum.workspace = true
tower.workspace = true
tower-http.workspace = true

# WebSocket
tokio-tungstenite.workspace = true

# File watching
notify.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Error handling
anyhow.workspace = true
thiserror.workspace = true

# Logging
tracing.workspace = true
tracing-subscriber.workspace = true

[dev-dependencies]
```

**Purpose**: Development server with HMR and live reload

**Dependencies**:
- `blang_driver` for compilation
- `tokio` for async runtime
- `axum` for HTTP server
- `notify` for file watching

---

## Dependency Summary

### Level 0 (No internal dependencies)
- `blang_common`

### Level 1 (Only depends on common)
- `blang_span` → common
- `blang_types` → common, span

### Level 2
- `blang_diagnostics` → common, span
- `blang_lexer` → common, span
- `blang_ast` → common, span
- `blang_runtime` → common

### Level 3
- `blang_parser` → common, span, diagnostics, lexer, ast
- `blang_resolve` → common, span, diagnostics, ast

### Level 4
- `blang_typeck` → common, span, diagnostics, ast, resolve, types
- `blang_ir` → common, span, ast, types

### Level 5
- `blang_borrow` → common, span, diagnostics, ir, types
- `blang_optimize` → common, ir, types
- `blang_component` → common, span, diagnostics, lexer, parser, ast
- `blang_script` → common, span, diagnostics, lexer, parser, ast

### Level 6
- `blang_codegen_wasm` → common, ir, types
- `blang_codegen_js` → common, ir, types

### Level 7
- `blang_driver` → all compiler crates

### Level 8
- `blang_cli` → driver, dev_server
- `blang_dev_server` → driver, diagnostics

---

## External Dependencies Summary

### Core Dependencies (used across many crates)

**rustc-hash** (1.1)
- Fast hash function (FxHash)
- Used in: common

**indexmap** (2.1)
- Ordered hash map
- Used in: resolve, types, typeck, ir, borrow, optimize, codegen_wasm, codegen_js, component, script, runtime

**smallvec** (1.11)
- Stack-allocated vectors
- Used in: ast, resolve, types, typeck, ir, borrow, optimize, codegen_wasm, component, script

**thiserror** (1.0)
- Error derive macro
- Used in: diagnostics, lexer, parser, resolve, typeck, borrow, codegen_wasm, codegen_js, component, script, driver, dev_server

**anyhow** (1.0)
- Flexible error handling
- Used in: codegen_wasm, codegen_js, driver, cli, dev_server

**tracing** (0.1)
- Structured logging
- Used in: parser, resolve, typeck, ir, borrow, optimize, codegen_wasm, codegen_js, component, script, driver, cli, dev_server

### WebAssembly Dependencies

**wasm-encoder** (0.38)
- WASM binary encoding
- Used in: codegen_wasm

**wasmparser** (0.118)
- WASM binary parsing/validation
- Used in: codegen_wasm

**walrus** (0.20)
- WASM manipulation
- Used in: codegen_wasm (dev dependencies)

### Web/WASM Runtime Dependencies

**wasm-bindgen** (0.2)
- JS/WASM interop
- Used in: runtime (WASM target only)

**js-sys** (0.3)
- JavaScript standard library bindings
- Used in: runtime (WASM target only)

**web-sys** (0.3)
- Web API bindings
- Used in: runtime (WASM target only)

### Dev Server Dependencies

**tokio** (1.35)
- Async runtime
- Used in: dev_server

**axum** (0.7)
- HTTP server framework
- Used in: dev_server

**notify** (6.1)
- File system watching
- Used in: dev_server

**tokio-tungstenite** (0.21)
- WebSocket support
- Used in: dev_server

### CLI Dependencies

**clap** (4.4)
- Command-line argument parsing
- Used in: cli

**colored** (2.1)
- Terminal colors
- Used in: diagnostics, cli

**indicatif** (0.17)
- Progress bars
- Used in: cli

### Testing Dependencies

**insta** (1.34)
- Snapshot testing
- Used in: lexer, ast, parser, resolve, typeck, ir, borrow, optimize, component, script (dev dependencies)

**proptest** (1.4)
- Property-based testing
- Used in: lexer, parser (dev dependencies)

**criterion** (0.5)
- Benchmarking
- Used in: common (dev dependencies)

---

## Build Order

To build the entire workspace from scratch:

```bash
# Build foundation crates first
cargo build -p blang_common
cargo build -p blang_span
cargo build -p blang_diagnostics
cargo build -p blang_types
cargo build -p blang_runtime

# Build frontend
cargo build -p blang_lexer
cargo build -p blang_ast
cargo build -p blang_parser

# Build middle-end
cargo build -p blang_resolve
cargo build -p blang_typeck
cargo build -p blang_ir
cargo build -p blang_borrow
cargo build -p blang_optimize

# Build backends
cargo build -p blang_codegen_wasm
cargo build -p blang_codegen_js

# Build mode-specific
cargo build -p blang_component
cargo build -p blang_script

# Build driver and tools
cargo build -p blang_driver
cargo build -p blang_dev_server
cargo build -p blang_cli

# Or build everything at once
cargo build --workspace
```

---

## Testing

```bash
# Test individual crate
cargo test -p blang_lexer

# Test all crates
cargo test --workspace

# Run benchmarks
cargo bench --workspace

# Check all crates
cargo check --workspace

# Clippy lints
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --workspace
```

---

## Notes

1. **Version Management**: All crates use workspace version (0.1.0)
2. **Edition**: All crates use Rust 2021 edition
3. **MSRV**: Minimum Supported Rust Version is 1.75
4. **License**: Dual-licensed MIT OR Apache-2.0
5. **Features**: Optional features are used sparingly and documented
6. **Dependencies**: Workspace dependencies ensure version consistency

---

This specification provides complete Cargo.toml configurations for all 20 crates in the Blang workspace, with clear dependency relationships and rationales for external dependencies.
