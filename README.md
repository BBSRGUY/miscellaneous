# Forge

A production-grade local LLM platform built with Rust and modern web technologies.

## Features

- 🚀 **Local-first**: Run LLMs entirely on your machine
- 🎯 **High Performance**: Built with Rust for maximum efficiency
- 🖥️ **Desktop App**: Native desktop experience via Tauri
- 🌐 **Web UI**: Modern React-based interface
- 🔧 **CLI**: Powerful command-line interface
- 🎨 **WebGPU**: Hardware-accelerated inference with WebGPU
- 📦 **Model Registry**: Manage and organize your models
- 💬 **Chat Sessions**: Persistent conversation history
- 🔬 **Fine-tuning**: LoRA/QLoRA training support
- 🔍 **RAG**: Document ingestion and retrieval

## Architecture

Forge is organized as a Rust workspace with the following crates:

- `forge_engine` - Model backends, inference, and training interfaces
- `forge_models` - Model registry, loading, and metadata
- `forge_runtime` - Sessions, pipelines, scheduler, and task execution
- `forge_store` - Database, migrations, and blob storage
- `forge_api` - HTTP/JSON API (Axum)
- `forge_cli` - Command-line interface
- `forge_tauri` - Desktop application shell
- `forge_wgpu_core` - Shared WebGPU kernels
- `forge_wasm` - WebAssembly bindings for browser

## Quick Start

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- Node.js 18+ (for UI development)
- Git

### Build from Source

```bash
# Clone the repository
git clone https://github.com/forge/forge.git
cd forge

# Build the workspace
cargo build

# Run the CLI
cargo run --bin forge -- --help

# Start the daemon
cargo run --bin forge -- daemon --port 3000
```

### UI Development

```bash
# Install dependencies
cd ui/app
npm install

# Start dev server
npm run dev
```

### Desktop App

```bash
# Build and run the desktop app
cargo tauri dev
```

## Development

### Project Structure

```
forge/
├── crates/              # Rust crates
│   ├── forge_engine/    # Inference and training
│   ├── forge_models/    # Model management
│   ├── forge_runtime/   # Task execution
│   ├── forge_store/     # Database layer
│   ├── forge_api/       # HTTP API
│   ├── forge_cli/       # CLI binary
│   ├── forge_tauri/     # Desktop app
│   ├── forge_wgpu_core/ # GPU kernels
│   └── forge_wasm/      # WASM bindings
├── ui/
│   └── app/            # React + TypeScript UI
├── configs/            # Configuration files
├── scripts/            # Development scripts
└── Cargo.toml          # Workspace manifest
```

### Scripts

- `scripts/dev.sh` - Build and show dev commands
- `scripts/test.sh` - Run all tests
- `scripts/build.sh` - Production build

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p forge_engine
```

## Configuration

Default configuration is in `configs/forge.toml`. You can override settings via:

1. Configuration file: `~/.config/forge/forge.toml`
2. Environment variables: `FORGE_*`

Example:

```toml
[daemon]
port = 3000
host = "127.0.0.1"

[storage]
database = "forge.db"
models_dir = "models"

[gpu]
enabled = true
```

## API

The Forge daemon exposes a REST API:

- `GET /health` - Health check
- `GET /status` - System status
- `GET /models` - List models
- `GET /sessions` - List sessions
- `POST /completions` - Generate completions

See the [API documentation](docs/api.md) for details.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Built with:

- [Rust](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/)
- [Axum](https://github.com/tokio-rs/axum)
- [Tauri](https://tauri.app/)
- [wgpu](https://wgpu.rs/)
- [React](https://react.dev/)
- [Vite](https://vitejs.dev/)
