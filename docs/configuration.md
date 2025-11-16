# Configuration Guide

Forge uses a layered configuration system that supports:

1. Default configuration from `configs/default.yaml`
2. Local overrides from `configs/local.yaml` (git-ignored)
3. Environment variables with `FORGE__` prefix

## Configuration File

The main configuration file is located at `configs/default.yaml`. To customize settings:

1. Copy `configs/local.yaml.example` to `configs/local.yaml`
2. Edit `local.yaml` with your custom settings
3. The `local.yaml` file is git-ignored and will override defaults

### Configuration Structure

```yaml
# API Server
api:
  host: "127.0.0.1"      # Host to bind to
  port: 3000              # HTTP port
  cors_enabled: true      # Enable CORS
  timeout_seconds: 30     # Request timeout

# Database
db:
  path: "forge.db"        # SQLite database file
  wal_mode: true          # Write-Ahead Logging
  pool_size: 5            # Connection pool size
  log_queries: false      # Enable query logging

# Storage
storage:
  data_dir: "data"              # Base data directory
  models_dir: "models"          # Models directory
  datasets_dir: "datasets"      # Datasets directory
  checkpoints_dir: "checkpoints" # Training checkpoints

# Logging
logging:
  level: "info"           # trace, debug, info, warn, error
  format: "pretty"        # json, pretty, compact
  file_enabled: false     # Enable file logging
  file_path: "logs/forge.log"
  console_enabled: true   # Enable console logging

# Inference
inference:
  max_concurrent_tasks: 4           # Max parallel tasks
  default_temperature: 0.7          # Generation temperature
  default_max_tokens: 2048          # Max tokens to generate
  enable_model_cache: true          # Cache loaded models

# GPU
gpu:
  enabled: true           # Enable GPU acceleration
  high_performance: true  # Prefer high-performance GPU
  webgpu_enabled: true    # Enable WebGPU support
```

## Environment Variables

You can override any configuration setting using environment variables with the `FORGE__` prefix. Use double underscores (`__`) to separate nested keys:

```bash
# Override API port
export FORGE__API__PORT=8080

# Override log level
export FORGE__LOGGING__LEVEL=debug

# Override database path
export FORGE__DB__PATH=/var/lib/forge/forge.db

# Override data directory
export FORGE__STORAGE__DATA_DIR=/data/forge
```

## CLI Configuration Options

The Forge CLI provides several flags to override configuration:

```bash
# Specify config directory
forge --config /path/to/configs daemon

# Enable verbose logging
forge --verbose config

# Enable debug logging
forge --debug config

# Override daemon settings
forge daemon --port 8080 --host 0.0.0.0
```

## Viewing Current Configuration

You can view the current configuration with:

```bash
# Human-readable format
forge config

# JSON format
forge config --json
```

## Logging

Forge uses structured logging with `tracing` and `tracing-subscriber`. You can configure logging via the configuration file or environment variables.

### Log Levels

- `trace`: Most verbose, includes all details
- `debug`: Detailed information for debugging
- `info`: General informational messages (default)
- `warn`: Warning messages for potentially harmful situations
- `error`: Error messages for failures

### Log Formats

- `pretty`: Human-readable colored logs with file/line info (development)
- `compact`: Single-line compact logs
- `json`: Structured JSON logs for production/parsing

### Examples

```bash
# Run with debug logging
forge --debug daemon

# Run with trace logging and JSON format
FORGE__LOGGING__LEVEL=trace FORGE__LOGGING__FORMAT=json forge daemon

# Run with custom log level from config
# Edit configs/local.yaml:
# logging:
#   level: debug
#   format: pretty
forge daemon
```

## Best Practices

1. **Never commit `local.yaml`**: Keep local settings in `local.yaml`, which is git-ignored
2. **Use environment variables for secrets**: Don't put secrets in config files
3. **Use relative paths**: Config paths are relative to the data directory
4. **Validate configuration**: Run `forge config` to verify settings before starting the daemon
5. **Production logging**: Use JSON format for production deployments for easier parsing

## Configuration Priority

Settings are loaded in this order (later overrides earlier):

1. `configs/default.yaml` (lowest priority)
2. `configs/local.yaml`
3. Environment variables
4. CLI flags (highest priority)

## Troubleshooting

### Configuration file not found

```
Error: Failed to load configuration: configuration file not found
```

Make sure you're running the command from the project root directory, or use the `--config` flag to specify the config directory.

### Invalid configuration

```
Error: Invalid configuration: API port must be non-zero
```

Check that all required fields are valid. Common issues:
- Port number is 0 or > 65535
- Invalid log level (must be trace/debug/info/warn/error)
- Invalid temperature (must be 0.0 - 2.0)

### Environment variable not working

Make sure you're using double underscores (`__`) to separate keys and the `FORGE__` prefix:

```bash
# Correct
FORGE__API__PORT=8080

# Incorrect
FORGE_API_PORT=8080
```
