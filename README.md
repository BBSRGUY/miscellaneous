# Forge - Local LLM Platform

[![CI](https://github.com/your-org/forge/workflows/CI/badge.svg)](https://github.com/your-org/forge/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A production-grade local LLM platform built with Rust and modern web technologies. Run large language models entirely on your machine with GPU acceleration, fine-tuning, and RAG capabilities.

## Features

- 🚀 **Local-first**: Run LLMs entirely on your machine with full privacy
- 🎯 **High Performance**: Built with Rust for maximum efficiency
- 🖥️ **Desktop App**: Native desktop experience via Tauri
- 🌐 **Web UI**: Modern React-based interface
- 🔧 **CLI**: Powerful command-line interface
- 🎨 **WebGPU**: Hardware-accelerated inference with WebGPU
- 📦 **Model Registry**: Manage and organize your models
- 💬 **Chat Sessions**: Persistent conversation history
- 🔬 **Fine-tuning**: LoRA/QLoRA training support
- 🔍 **RAG**: Document ingestion and retrieval
- 🔒 **Secure**: Localhost-only API, input validation, CORS restrictions

## Table of Contents

- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
  - [Starting the Daemon](#starting-the-daemon)
  - [Using the CLI](#using-the-cli)
  - [Using the Desktop App](#using-the-desktop-app)
  - [Running Training](#running-training)
  - [Using RAG](#using-rag)
- [Development](#development)
- [Configuration](#configuration)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [License](#license)

## Architecture

Forge is organized as a modular Rust workspace with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interfaces                         │
├──────────────┬──────────────────────┬──────────────────────┤
│   CLI Tool   │    Tauri Desktop     │     Web Browser      │
│  (forge_cli) │   (forge_tauri)      │   (React + Vite)     │
└──────┬───────┴──────────┬───────────┴──────────┬───────────┘
       │                  │                      │
       └──────────────────┴──────────────────────┘
                          │
                  ┌───────▼────────┐
                  │   HTTP API     │
                  │  (forge_api)   │
                  │  Port: 3000    │
                  └───────┬────────┘
                          │
          ┌───────────────┴────────────────┐
          │         Runtime Layer          │
          │       (forge_runtime)          │
          │  • Sessions Management         │
          │  • Task Scheduler              │
          │  • RAG Service                 │
          │  • Training Jobs               │
          └───────┬────────────────────────┘
                  │
    ┌─────────────┼─────────────┬──────────────┐
    │             │             │              │
┌───▼────┐  ┌────▼─────┐  ┌───▼────┐   ┌────▼─────┐
│ Engine │  │  Models  │  │ Store  │   │  WebGPU  │
│        │  │ Registry │  │        │   │  Kernels │
│ Infer  │  │  Loader  │  │ SQLite │   │  (wgpu)  │
│ Train  │  │  Meta    │  │ Blobs  │   │          │
└────────┘  └──────────┘  └────────┘   └──────────┘
```

### Crate Overview

| Crate | Purpose | Key Responsibilities |
|-------|---------|---------------------|
| `forge_engine` | Inference & Training | Model execution, streaming, LoRA/QLoRA |
| `forge_models` | Model Management | Registry, loading, metadata |
| `forge_runtime` | Orchestration | Sessions, scheduler, RAG, jobs |
| `forge_store` | Persistence | SQLite database, blob storage |
| `forge_api` | HTTP Interface | REST API with Axum |
| `forge_cli` | Command Line | CLI tool for users |
| `forge_tauri` | Desktop App | Native app wrapper |
| `forge_wgpu_core` | GPU Acceleration | WebGPU compute kernels |
| `forge_wasm` | Browser Support | WebAssembly bindings |

## Quick Start

### Prerequisites

- **Rust**: 1.75 or later (install from [rustup.rs](https://rustup.rs))
- **Node.js**: 18 or later (for UI development)
- **Git**: For cloning the repository

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-org/forge.git
cd forge

# Build the workspace (this may take a few minutes)
cargo build --release

# Verify installation
./target/release/forge --version
```

### Quick Test

```bash
# Start the daemon in one terminal
cargo run --release --bin forge -- serve

# In another terminal, use the CLI
cargo run --release --bin forge -- models list
cargo run --release --bin forge -- chat
```

## Installation

### Option 1: Pre-built Binaries (Recommended)

Download pre-built binaries from the [Releases](https://github.com/your-org/forge/releases) page:

- **Linux**: `forge-vX.Y.Z-linux-x64.tar.gz`
- **macOS**: `forge-vX.Y.Z-macos-x64.tar.gz` or `forge-vX.Y.Z-macos-arm64.tar.gz`
- **Windows**: `forge-vX.Y.Z-windows-x64.zip`

Extract and add to your PATH:

```bash
# Linux/macOS
tar -xzf forge-*.tar.gz
export PATH="$PATH:$PWD/forge-*/bin"

# Windows (PowerShell)
Expand-Archive forge-*.zip
$env:PATH += ";$PWD\forge-*\bin"
```

### Option 2: Build from Source

```bash
# Clone and build
git clone https://github.com/your-org/forge.git
cd forge
cargo build --release

# Install to system (optional)
cargo install --path crates/forge_cli
```

### Option 3: Desktop App

Download the desktop app installer:

- **Linux**: `.deb` or `.AppImage`
- **macOS**: `.dmg`
- **Windows**: `.msi` or `.exe` installer

## Usage

### Starting the Daemon

The Forge daemon provides the HTTP API that powers the CLI, desktop app, and web UI.

```bash
# Start with default settings (localhost:3000)
forge serve

# Custom port and host
forge serve --port 8080 --host 127.0.0.1

# With custom config
forge serve --config /path/to/config.yaml

# Verbose logging
forge serve --verbose
```

The daemon binds to `127.0.0.1` by default for security. To access from other machines on your network:

```bash
# WARNING: Only do this on trusted networks
forge serve --host 0.0.0.0
```

### Using the CLI

#### Interactive Chat

```bash
# Start an interactive chat session
forge chat

# Chat with a specific model
forge chat --model llama2-7b

# Resume a previous session
forge chat --session-id abc-123
```

#### Model Management

```bash
# List available models
forge models list

# Add a new model
forge models add \
  --name "llama2-7b" \
  --path "/path/to/model.gguf" \
  --backend llama \
  --format gguf

# Load a model into memory
forge models load llama2-7b

# Unload a model
forge models unload llama2-7b
```

#### Session Management

```bash
# List chat sessions
forge sessions list

# View session details
forge sessions show abc-123

# Delete a session
forge sessions delete abc-123
```

#### Job Tracking

```bash
# List background jobs
forge jobs list

# View job status
forge jobs status job-456

# Cancel a running job
forge jobs cancel job-456
```

### Using the Desktop App

1. **Launch the app**: Double-click the Forge icon or run from the command line:
   ```bash
   # macOS
   open -a Forge

   # Linux
   forge-tauri

   # Windows
   forge-tauri.exe
   ```

2. **First-time setup**:
   - The app will automatically start the daemon on launch
   - Wait for the "Ready" indicator in the status bar

3. **Chat Interface**:
   - Select a model from the dropdown
   - Type your message in the input box
   - Press Enter or click Send
   - Responses stream in real-time

4. **Model Management**:
   - Click "Models" in the sidebar
   - Add, load, or remove models
   - View model details and memory usage

5. **Jobs & Training**:
   - Click "Jobs" to see active and completed jobs
   - Monitor training progress in real-time
   - View logs and metrics

### Running Training

Forge supports LoRA and QLoRA fine-tuning for model customization.

#### Prepare Training Data

Create a JSONL file with training examples:

```jsonl
{"prompt": "What is Rust?", "completion": "Rust is a systems programming language..."}
{"prompt": "Explain ownership", "completion": "Ownership is Rust's approach to memory safety..."}
```

#### Start Training via CLI

```bash
# Basic training
forge train \
  --model llama2-7b \
  --dataset training_data.jsonl \
  --output models/llama2-7b-finetuned

# Advanced configuration
forge train \
  --model llama2-7b \
  --dataset training_data.jsonl \
  --output models/llama2-7b-finetuned \
  --epochs 3 \
  --learning-rate 1e-4 \
  --lora-rank 16 \
  --lora-alpha 32 \
  --batch-size 4
```

#### Monitor Training

```bash
# List training jobs
forge jobs list --type training

# View training logs
forge jobs logs job-789

# Check training status
forge jobs status job-789
```

#### Training Configuration

Available parameters:

- `--epochs`: Number of training epochs (default: 3)
- `--learning-rate`: Learning rate (default: 1e-4)
- `--lora-rank`: LoRA rank (default: 8)
- `--lora-alpha`: LoRA alpha (default: 16)
- `--batch-size`: Training batch size (default: 1)
- `--gradient-accumulation-steps`: Gradient accumulation (default: 1)
- `--max-grad-norm`: Max gradient norm for clipping (default: 1.0)

### Using RAG

Retrieval-Augmented Generation (RAG) enhances responses with external knowledge.

#### Ingest Documents

```bash
# Add a single document
forge rag add document.txt

# Add a directory of documents
forge rag add --recursive ./docs/

# Supported formats: .txt, .md, .pdf, .html
```

#### Query with RAG

```bash
# Chat with RAG enabled
forge chat --rag

# Query specific documents
forge rag query "What is the pricing model?" --limit 5
```

#### Manage Documents

```bash
# List ingested documents
forge rag list

# View document details
forge rag show doc-id-123

# Delete a document
forge rag delete doc-id-123

# Re-index documents
forge rag reindex
```

#### RAG Configuration

Configure in `configs/default.yaml`:

```yaml
rag:
  chunk_size: 512
  chunk_overlap: 50
  embedding_model: "all-MiniLM-L6-v2"
  similarity_top_k: 5
```

## Development

### Project Structure

```
forge/
├── crates/              # Rust crates
│   ├── forge_api/       # HTTP API (Axum)
│   ├── forge_cli/       # CLI binary
│   ├── forge_engine/    # Inference and training
│   ├── forge_models/    # Model management
│   ├── forge_runtime/   # Task execution
│   ├── forge_store/     # Database layer
│   ├── forge_tauri/     # Desktop app
│   ├── forge_wasm/      # WASM bindings
│   └── forge_wgpu_core/ # GPU kernels
├── ui/
│   └── app/            # React + TypeScript UI
├── configs/            # Configuration files
├── scripts/            # Build and development scripts
├── .github/            # GitHub Actions CI/CD
└── Cargo.toml          # Workspace manifest
```

### Development Workflow

```bash
# Build all crates
cargo build

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build UI
cd ui/app && npm install && npm run build

# Run dev server
cd ui/app && npm run dev
```

### Running Tests

```bash
# Run all tests
cargo test --workspace --verbose

# Run specific crate tests
cargo test -p forge_store
cargo test -p forge_engine
cargo test -p forge_runtime

# Run with coverage
cargo tarpaulin --workspace --out html

# Run integration tests only
cargo test --test '*'
```

### Scripts

Located in `scripts/`:

- `build.sh` - Production build for all components
- `dev.sh` - Development build
- `test.sh` - Run all tests
- `package-release.sh` - Package release binaries
- `package-release.ps1` - Package release binaries (Windows)

### Development Tools

```bash
# Install development dependencies
cargo install cargo-watch      # Auto-rebuild on changes
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit      # Security audit

# Watch and rebuild on changes
cargo watch -x build

# Security audit
cargo audit
```

## Configuration

### Configuration Files

Forge uses a hierarchical configuration system:

1. **Default**: `configs/default.yaml` (in repository)
2. **Local override**: `configs/local.yaml` (git-ignored)
3. **Environment variables**: `FORGE__` prefix

### Default Configuration

Located in `configs/default.yaml`:

```yaml
# API Server
api:
  host: "127.0.0.1"      # Bind to localhost only (secure)
  port: 3000
  cors_enabled: true
  timeout_seconds: 30

# Database
db:
  path: "forge.db"
  wal_mode: true
  pool_size: 5

# Storage
storage:
  data_dir: "data"
  models_dir: "models"
  datasets_dir: "datasets"
  checkpoints_dir: "checkpoints"

# Logging
logging:
  level: "info"         # trace, debug, info, warn, error
  format: "pretty"      # json, pretty, compact
  file_enabled: false
  console_enabled: true

# Inference
inference:
  max_concurrent_tasks: 4
  default_temperature: 0.7
  default_max_tokens: 2048

# GPU
gpu:
  enabled: true
  high_performance: true
  webgpu_enabled: true
```

### Environment Variables

Override any config value with environment variables:

```bash
# Format: FORGE__SECTION__KEY=value
export FORGE__API__PORT=8080
export FORGE__LOGGING__LEVEL=debug
export FORGE__GPU__ENABLED=false

forge serve
```

### Creating a Local Config

```bash
# Copy example config
cp configs/default.yaml configs/local.yaml

# Edit local.yaml with your settings
# This file is git-ignored
```

## API Reference

The Forge API provides RESTful endpoints for all functionality.

### Base URL

```
http://localhost:3000/api/v1
```

### Authentication

Currently, Forge uses localhost-only access for security. Authentication is planned for future releases.

### Endpoints

#### Health & Status

```bash
# Health check
GET /health

# System status
GET /config
```

#### Models

```bash
# List models
GET /models

# Register a model
POST /models
Body: {
  "name": "llama2-7b",
  "path": "/path/to/model.gguf",
  "backend": "llama",
  "format": "gguf"
}

# Load a model
POST /models/:id/load
```

#### Chat

```bash
# Send a chat message
POST /chat
Body: {
  "model_id": "llama2-7b",
  "prompt": "Hello!",
  "stream": true
}
```

#### Sessions

```bash
# List sessions
GET /sessions

# Get session details
GET /sessions/:id
```

#### Training

```bash
# Create training job
POST /train/jobs
Body: {
  "model_id": "llama2-7b",
  "dataset_path": "training_data.jsonl",
  "config": {
    "epochs": 3,
    "learning_rate": 1e-4
  }
}

# Get training job status
GET /train/jobs/:id

# Cancel training
POST /train/jobs/:id/cancel

# Get training logs
GET /train/jobs/:id/logs
```

#### RAG

```bash
# Ingest document
POST /rag/documents
Body: {
  "source": "document.txt",
  "content": "Document text..."
}

# List documents
GET /rag/documents

# Query RAG
POST /rag/query
Body: {
  "query": "What is Rust?",
  "top_k": 5
}

# Delete document
DELETE /rag/documents/:id
```

### Error Responses

All errors follow this format:

```json
{
  "error": "Error message",
  "status": 400
}
```

Status codes:
- `200` - Success
- `400` - Bad Request (validation failed)
- `404` - Not Found
- `500` - Internal Server Error

## Security

Forge implements multiple security layers:

1. **Localhost Binding**: API binds to `127.0.0.1` by default
2. **CORS Restrictions**: Only allows local origins
3. **Input Validation**: All user inputs are validated and sanitized
4. **Log Sanitization**: Prevents log injection attacks
5. **Path Validation**: Prevents directory traversal
6. **No External Network**: All operations are local

### Security Best Practices

- Keep the daemon running on localhost only
- Don't expose the API to untrusted networks
- Regularly update dependencies (`cargo update`)
- Run security audits (`cargo audit`)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

```bash
# Fork and clone
git clone https://github.com/your-username/forge.git
cd forge

# Create a branch
git checkout -b feature/amazing-feature

# Make changes and test
cargo test --workspace
cargo clippy --all-targets

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/amazing-feature

# Open a Pull Request
```

### Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Roadmap

- [x] Core inference engine
- [x] Model registry and management
- [x] HTTP API with streaming
- [x] Desktop application
- [x] WebGPU acceleration
- [x] LoRA/QLoRA fine-tuning
- [x] RAG support
- [x] CI/CD pipeline
- [ ] Authentication and authorization
- [ ] Multi-user support
- [ ] Plugin system
- [ ] Additional model backends (ONNX, TensorRT)
- [ ] Mobile apps (iOS/Android)
- [ ] Distributed inference

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

Forge is built with excellent open-source technologies:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tauri](https://tauri.app/) - Desktop app framework
- [wgpu](https://wgpu.rs/) - GPU acceleration
- [SQLite](https://www.sqlite.org/) - Database
- [React](https://react.dev/) - UI library
- [Vite](https://vitejs.dev/) - Build tool

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/forge/discussions)
- **Documentation**: [docs/](docs/)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes.
