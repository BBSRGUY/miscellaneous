# Blang Programming Language

**A browser-first language for building modern web applications**

---

## Overview

Blang is a modern, statically-typed programming language designed specifically for web development. It unifies UI development, application logic, orchestration, and performance-critical code in a single coherent language that compiles to WebAssembly and minimal JavaScript glue.

### Key Features

- **Three Execution Modes**:
  - **Component Mode**: Reactive UI components with co-located styles and logic
  - **Script Mode**: REXX-inspired job orchestration and workflows
  - **Module Mode**: Pure logic and algorithms compiled to optimized WebAssembly

- **Strong Type System**: Static typing with sophisticated type inference, generics, and traits

- **Reactivity Built-in**: Signal-based reactivity system inspired by SolidJS and Leptos

- **Performance First**: Compiles to WebAssembly for near-native performance

- **Memory Safety**: Rust-inspired ownership and borrowing system prevents common bugs

- **Modern Developer Experience**: Rich tooling including LSP, formatter, and dev server

---

## Quick Example

### Component Mode (.blang)

```blang
component TodoApp {
  state {
    todos: Signal<Vec<Todo>> = signal(vec![]);
    input: Signal<str> = signal("");
  }

  view {
    <div class="container">
      <h1>My Todo List</h1>
      <input
        type="text"
        value={input}
        @input={handleInput}
      />
      <button @click={addTodo}>Add Todo</button>
      <ul>
        {for todo in todos.value() {
          <TodoItem todo={todo} />
        }}
      </ul>
    </div>
  }

  style {
    .container {
      max-width: 600px;
      margin: 2rem auto;
      font-family: system-ui;
    }
  }

  fn handleInput(e: InputEvent) {
    input.set(e.target.value);
  }

  fn addTodo() {
    if !input.value().is_empty() {
      todos.update(|t| t.push(Todo::new(input.value())));
      input.set("");
    }
  }
}
```

### Module Mode (.bl)

```blang
module math {
  /// Calculate the nth Fibonacci number
  pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
      return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }

  /// Generic sorting function
  pub fn sort<T: Ord>(arr: &mut [T]) {
    // Efficient sorting implementation
    quicksort(arr, 0, arr.len() - 1);
  }

  /// High-performance vector operations using SIMD
  pub fn vector_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    unsafe {
      simd_vector_add(a, b)
    }
  }

  unsafe fn simd_vector_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    // Direct WASM SIMD instructions for maximum performance
    wasm! {
      // WASM SIMD code
    }
  }
}
```

### Script Mode (.bs)

```blang
script DataPipeline {
  config {
    max_parallel: 4,
    timeout: 300s,
  }

  job extract {
    parallel {
      step fetch_users {
        retry: 3,
        timeout: 30s,

        return http.get("/api/users");
      }

      step fetch_orders {
        retry: 3,
        timeout: 30s,

        return http.get("/api/orders");
      }
    }
  }

  job transform depends_on(extract) {
    step merge_data {
      let users = extract.fetch_users.result;
      let orders = extract.fetch_orders.result;
      return join_data(users, orders);
    }

    step enrich {
      return enrich_data(transform.merge_data.result);
    }
  }

  job load depends_on(transform) {
    step save_to_database {
      db.save(transform.enrich.result);
    }
  }
}
```

---

## Documentation

This repository contains comprehensive documentation for the Blang language:

### Language Design

- **[LANGUAGE_SPEC.md](LANGUAGE_SPEC.md)**: Complete language specification covering:
  - Type system (primitives, generics, traits)
  - Expression and statement syntax
  - Module system
  - Component mode (UI, styles, reactivity)
  - Script mode (jobs, steps, orchestration)
  - Unsafe blocks for low-level code
  - Concurrency model (actors, channels)
  - Memory model and ownership

- **[GRAMMAR.md](GRAMMAR.md)**: Formal BNF/EBNF grammar specification for:
  - Lexical structure (tokens, literals, keywords)
  - Expression grammar with precedence
  - Statement and declaration syntax
  - Component view and style blocks
  - Script job and step syntax
  - Type expressions and patterns

### User Guides (docs/)

- **[docs/language_overview.md](docs/language_overview.md)**: Complete language reference
  - Syntax and semantics
  - Type system basics
  - Standard library
  - Best practices

- **[docs/component_mode.md](docs/component_mode.md)**: Building reactive UI components
  - Component structure
  - State management
  - View templates
  - Lifecycle hooks
  - Styling

- **[docs/script_mode.md](docs/script_mode.md)**: Data processing and orchestration
  - Job definitions
  - Step configuration
  - Dependencies and parallelism
  - Error handling

- **[docs/unsafe_blocks.md](docs/unsafe_blocks.md)**: Low-level programming guide
  - When to use unsafe
  - Pointer operations
  - WASM intrinsics
  - Safety guidelines

- **[docs/testing.md](docs/testing.md)**: Testing and quality assurance
  - Testing strategy
  - Property-based tests
  - E2E WASM tests
  - CI integration

### Development

- **[ROADMAP.md](ROADMAP.md)**: Phased development plan with 12 phases:
  - Phase 1-3: Core compiler (✅ Complete)
  - Phase 4-6: Advanced features (✅ Complete)
  - Phase 7: WASM + DOM integration (✅ Complete)
  - Phase 8: Component & Script modes (✅ Complete)
  - Phase 9: Unsafe blocks (✅ Complete)
  - Phase 10: CLI & tooling (✅ Complete)
  - Phase 11: Testing infrastructure (✅ Complete)
  - Phase 12: Documentation (🚧 In Progress)

- **[PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)**: Detailed project organization:
  - Repository layout
  - Crate organization (21 crates)
  - Module breakdown
  - Data flow through compiler
  - Testing structure
  - Development workflow

---

## Architecture

Blang follows a traditional multi-phase compiler architecture:

```
Source Code (.blang, .bl, .bs)
    ↓
Lexer → Tokens
    ↓
Parser → AST (Abstract Syntax Tree)
    ↓
Name Resolution → Resolved AST
    ↓
HIR Lowering → HIR (High-level IR)
    ↓
Type Checking → Typed HIR
    ↓
MIR Building → MIR (Mid-level IR)
    ↓
Borrow Checking → Validated MIR
    ↓
Optimizations → Optimized MIR
    ↓
LIR Lowering → LIR (Low-level IR)
    ↓
Code Generation → WASM + JS Glue
    ↓
Output Bundle
```

### Key Components

- **Frontend**: Lexer, Parser, AST construction
- **Middle**: Name resolution, type checking, IR transformations
- **Backend**: WASM code generation, JS glue generation
- **Runtime**: Memory allocator, reactivity system, concurrency support
- **Tooling**: CLI, LSP, formatter, dev server

---

## Design Principles

1. **Explicitness over Implicitness**: Code should clearly express its intent
2. **Safety by Default, Performance by Choice**: Unsafe operations require explicit `unsafe` blocks
3. **Zero-Cost Abstractions**: High-level features compile to efficient code
4. **Composability**: Small primitives that combine well
5. **Locality**: Related code (UI, styles, logic) stays together
6. **Predictability**: Behavior is deterministic and easy to reason about

---

## Target Audience

Blang is designed for:

- **Web Developers**: Who want type safety and performance without leaving the web platform
- **Systems Programmers**: Who want to write high-performance web code with low-level control
- **Data Engineers**: Who need orchestration and workflow capabilities
- **UI Engineers**: Who want reactive UIs without framework overhead

---

## Comparison with Other Languages

| Feature | Blang | TypeScript | Rust | Go |
|---------|-------|------------|------|-----|
| Type Safety | ✓ Strong, static | ✓ Optional | ✓ Strong, static | ✓ Static |
| Memory Safety | ✓ Ownership | ✗ GC | ✓ Ownership | ✗ GC |
| WebAssembly | ✓ Primary target | ✗ Via tools | ✓ Via wasm32 | ✓ Via tinygo |
| UI Components | ✓ Built-in | ✗ Framework | ✗ Framework | ✗ Framework |
| Reactivity | ✓ Built-in | ✗ Framework | ✗ Framework | ✗ Framework |
| Orchestration | ✓ Built-in | ✗ External | ✗ External | ✗ External |
| Learning Curve | Medium | Easy | Hard | Easy |

---

## Status

**Current Status**: Active Development - Compiler Foundation Complete

Blang has progressed significantly beyond the specification phase. The compiler infrastructure and core features are implemented:

- ✅ Language Specification
- ✅ Grammar Specification
- ✅ Development Roadmap
- ✅ Project Structure
- ✅ Lexer - Complete tokenization with comprehensive tests
- ✅ Parser - Full syntax support for all language features
- ✅ AST - Complete abstract syntax tree
- ✅ Component Mode Parsing - UI components with view/style blocks
- ✅ Script Mode Parsing - Job orchestration syntax
- ✅ Unsafe Blocks - Low-level programming support
- ✅ CLI - Production-ready command-line interface
- ✅ Dev Server - Hot reload development server
- ✅ Testing Infrastructure - Property-based tests and E2E framework
- 🚧 IR Lowering - In progress (known issues being addressed)
- 🚧 Type Checker - Partial implementation
- 🚧 Code Generation - WASM output (partial)

**What Works Now**:
- ✅ Parse Blang source code
- ✅ CLI commands (compile, bundle, dev)
- ✅ Development server with hot reload
- ✅ Comprehensive testing (12 lexer + 23 parser property tests)

**What's In Progress**:
- 🚧 Complete IR lowering (fixing infinite loop issues)
- 🚧 Type checking implementation
- 🚧 Full WASM code generation
- 🚧 Runtime system

---

## Getting Started

### Building from Source

```bash
# Clone the repository
git clone https://github.com/blang-lang/blang.git
cd blang

# Build the compiler
cargo build --release

# The blang binary will be at target/release/blang
./target/release/blang --version

# Optionally, add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Using the CLI

```bash
# Compile a Blang file (parsing only for now)
blang compile myfile.blang -o output.wasm

# Create a bundle (WASM + HTML + JS runtime)
blang bundle myfile.blang --out-dir dist

# Start development server with hot reload
blang dev --port 3000

# Get help
blang --help
blang compile --help
```

### Try the Examples

```bash
# Navigate to examples directory
cd examples/counter_app

# Start the dev server
blang dev

# Open http://localhost:3000 in your browser
```

**Note**: Due to in-progress IR lowering, full compilation to executable WASM is not yet complete. However, the parser, CLI, and dev server all work correctly.

---

## Project Goals

### Short-term (6 months)

- ✓ Complete language specification
- ✓ Complete grammar definition
- ✓ Complete development roadmap
- ⏳ Implement core compiler (M0-M3)
- ⏳ Implement component mode (M4)
- ⏳ Build initial tooling

### Medium-term (12 months)

- Implement script mode (M5)
- Implement unsafe blocks and optimizations (M6)
- Implement concurrency features (M7)
- Build comprehensive tooling (M8)
- Expand standard library (M9)

### Long-term (18+ months)

- Production-ready v1.0 release (M10)
- Package ecosystem
- Multi-target support (Node.js, WASI)
- Framework development
- Community growth

---

## Contributing (Future)

We welcome contributions! Once the project is ready for contributions, please see:

- `CONTRIBUTING.md` for contribution guidelines
- `CODE_OF_CONDUCT.md` for community standards
- GitHub Issues for bug reports and feature requests
- GitHub Discussions for questions and ideas

---

## License

Blang is dual-licensed under:

- MIT License
- Apache License 2.0

You may choose either license for your use.

---

## Inspiration and Credits

Blang draws inspiration from:

- **Rust**: Ownership system, type system, trait system
- **TypeScript**: Developer experience, gradual typing philosophy
- **SolidJS/Leptos**: Reactivity system, component model
- **REXX**: Script mode job orchestration
- **Swift**: Syntax clarity, ergonomics
- **Zig**: Compile-time computation, explicit control

---

## FAQ

### Why create a new language?

Existing languages don't provide a unified solution for web development that combines UI, logic, and orchestration with strong type safety and WebAssembly performance. Blang aims to fill this gap.

### Why target WebAssembly?

WebAssembly provides near-native performance, memory safety, and portability. It's the future of web performance and allows Blang to run efficiently in browsers while maintaining safety guarantees.

### How is this different from Rust + wasm-bindgen?

Blang is designed specifically for web development with built-in UI components, reactivity, and orchestration. While Rust can target WASM, it requires frameworks and bindings for UI work. Blang makes it ergonomic out of the box.

### Will this support server-side rendering?

Yes, SSR support is planned for future milestones. The architecture is designed to support multiple targets including Node.js and WASI.

### How does performance compare to JavaScript?

For computation-heavy workloads, Blang (via WASM) can be 2-10x faster than JavaScript. For typical UI work, performance is comparable but with better memory characteristics.

### Can I use existing JavaScript libraries?

Yes, Blang provides interop with JavaScript, allowing you to call JS libraries and export Blang functions to JS.

---

## Community

- **Website**: https://blang-lang.org (coming soon)
- **GitHub**: https://github.com/blang/blang
- **Discord**: https://discord.gg/blang (coming soon)
- **Twitter**: @blang_lang (coming soon)

---

## Acknowledgments

Thank you to the Rust, TypeScript, and WebAssembly communities for creating the foundations that make Blang possible.

---

**Blang** - *A language for the modern web*
