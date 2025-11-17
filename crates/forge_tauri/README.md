# Forge Tauri - Desktop Application

Desktop shell for the Forge platform using Tauri with embedded terminal and web UI.

## Features

- **Automatic Daemon Management**: Starts Forge daemon (HTTP API server) on application startup
- **Split Layout Interface**:
  - **Left Panel**: Embedded terminal using xterm.js (placeholder for future PTY integration)
  - **Right Panel**: Web UI with tabs for Chat, Models, and Jobs
- **Real-time API Status**: Health check indicator with automatic reconnection
- **Tab Navigation**: Switch between Chat, Models, and Jobs panels
- **Models Management**: View registered models with details
- **Jobs Monitoring**: Track active jobs with auto-refresh

## Architecture

### Backend (Rust)

The Tauri backend (`src/main.rs`) provides commands to:

1. **Daemon Management**:
   - `start_daemon()` - Start the Forge HTTP API server
   - `stop_daemon()` - Stop the running daemon
   - `check_api_health()` - Verify API connectivity

2. **API Interaction**:
   - `list_models()` - Fetch all registered models
   - `list_jobs()` - Fetch all active jobs

3. **Terminal (Placeholder)**:
   - `spawn_terminal()` - Spawn terminal process
   - `terminal_input()` - Send input to terminal
   - `terminal_resize()` - Resize terminal dimensions

All API interaction commands use HTTP client to communicate with the Forge API,
ensuring Tauri and CLI share the same interface.

### Frontend (React + TypeScript)

The frontend is built with:
- **React 18** for UI components
- **TypeScript** for type safety
- **Vite** for fast development and building
- **xterm.js** for terminal emulation
- **@tauri-apps/api** for backend communication

#### Components:

- `App.tsx` - Main application with layout and navigation
- `Terminal.tsx` - Embedded terminal with xterm.js
- `ChatPanel.tsx` - Chat interface (minimal implementation)
- `ModelsPanel.tsx` - Models listing and management
- `JobsPanel.tsx` - Jobs monitoring with auto-refresh

## Prerequisites

### System Dependencies (Linux)

Tauri requires system libraries for GTK development:

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel

# Arch
sudo pacman -S webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  libappindicator-gtk3 \
  librsvg
```

### Development Tools

- **Rust**: 1.70+
- **Node.js**: 18+
- **npm**: 9+

## Setup

### 1. Install Frontend Dependencies

```bash
cd ui/app
npm install
```

### 2. Build the Forge CLI

The Tauri app depends on the `forge` CLI binary:

```bash
# From repository root
cargo build -p forge_cli
```

This creates `target/debug/forge` which the Tauri app uses to start the daemon.

### 3. Ensure Configuration Exists

Make sure you have a configuration file at `configs/default.yaml`:

```yaml
api:
  host: "127.0.0.1"
  port: 3000
  cors_enabled: true

db:
  wal_mode: true
  pool_size: 5

storage:
  data_dir: "data"

logging:
  level: "info"
  format: "compact"

inference:
  max_concurrent_tasks: 4
  default_temperature: 0.7
  default_max_tokens: 2048

gpu:
  enabled: false
  high_performance: false
```

## Development

### Run in Development Mode

```bash
# Terminal 1: Build frontend and watch for changes
cd ui/app
npm run dev

# Terminal 2: Run Tauri dev mode (from repository root)
cd crates/forge_tauri
cargo tauri dev
```

The Tauri dev mode will:
1. Start the Forge daemon automatically
2. Open the desktop application window
3. Hot-reload frontend changes

### Build Frontend for Production

```bash
cd ui/app
npm run build
```

This creates optimized assets in `ui/app/dist/` which Tauri bundles into the application.

## Building

### Build Development Binary

```bash
cd crates/forge_tauri
cargo tauri build --debug
```

### Build Production Binary

```bash
cd crates/forge_tauri
cargo tauri build
```

The built application will be in `target/release/bundle/`.

## Usage

### Starting the Application

When you run the Tauri application:

1. **Auto-start**: The app automatically starts the Forge daemon on launch
2. **Health Check**: Top bar shows API status (green = healthy, red = offline)
3. **Manual Start**: If daemon fails to start, click "Start Daemon" button

### Using the Interface

**Left Panel - Terminal**:
- Placeholder terminal with basic commands
- Type `help` for available commands
- In future, will support full PTY with forge CLI

**Right Panel - Web UI**:
- **Chat Tab**: Placeholder for chat interface
- **Models Tab**: View all registered models with refresh button
- **Jobs Tab**: Monitor active jobs with auto-refresh (every 5 seconds)

### Example Workflow

1. Start the Tauri app
2. Wait for API status to show "healthy"
3. Navigate to "Models" tab
4. Use the terminal (or external CLI) to register a model:
   ```bash
   forge models add --name test-model --path ./model.gguf --backend echo
   ```
5. Click "Refresh" in Models tab to see the new model
6. Switch to "Jobs" tab to monitor any running tasks

## Terminal Integration (Future)

The current terminal is a placeholder. Full PTY integration would require:

1. **Backend**: Use a PTY library like `portable-pty` to spawn real shell processes
2. **IPC**: Stream stdout/stderr over Tauri events to frontend
3. **Input**: Send terminal input from frontend to PTY stdin
4. **Resize**: Handle terminal resize events properly

Placeholder commands:
- `help` - Show available commands
- `clear` - Clear terminal
- `status` - Check API health

## Troubleshooting

### Daemon Fails to Start

- Ensure `target/debug/forge` exists (run `cargo build -p forge_cli`)
- Check that port 3000 is not in use
- Verify `configs/default.yaml` exists and is valid

### Frontend Not Loading

- Run `cd ui/app && npm install && npm run build`
- Check that `ui/app/dist/` contains built assets
- Verify `tauri.conf.json` points to correct `frontendDist` path

### API Connection Errors

- Check the terminal output for daemon logs
- Verify daemon is running: `curl http://localhost:3000/api/v1/health`
- Ensure CORS is enabled in configuration

## Project Structure

```
crates/forge_tauri/
├── src/
│   └── main.rs              # Tauri backend with commands
├── Cargo.toml               # Rust dependencies
├── tauri.conf.json          # Tauri configuration
├── build.rs                 # Build script
└── README.md                # This file

ui/app/
├── src/
│   ├── components/
│   │   ├── Terminal.tsx     # xterm.js terminal
│   │   ├── ChatPanel.tsx    # Chat interface
│   │   ├── ModelsPanel.tsx  # Models management
│   │   └── JobsPanel.tsx    # Jobs monitoring
│   ├── App.tsx              # Main application
│   ├── App.css              # Styles
│   └── main.tsx             # Entry point
├── package.json             # Node dependencies
├── vite.config.ts           # Vite configuration
└── tsconfig.json            # TypeScript config
```

## Development Notes

- All backend-frontend communication uses Tauri's `invoke` API
- API calls are made via HTTP client, not direct module imports
- This ensures CLI and Tauri share the same HTTP API interface
- Terminal currently uses basic command handling; full PTY integration planned
- Frontend uses dark theme matching VS Code aesthetic

## License

See repository root for license information.
