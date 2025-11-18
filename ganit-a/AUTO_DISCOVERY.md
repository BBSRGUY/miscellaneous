# Auto-Discovery Mode

**Continuous Mathematical Exploration with Checkpointing**

## 🚀 Overview

Auto-Discovery Mode allows Ganit-A to run **continuously**, exploring the mathematical landscape indefinitely while:

- **Resuming from checkpoints** - Never lose progress
- **Tracking exploration state** - Avoid re-exploring parameters
- **Graceful shutdown** - Stop/pause/resume anytime
- **Real-time progress** - Monitor via CLI or Web UI
- **Persistent logs** - Full audit trail of discoveries

---

## 🎯 Use Cases

### Research & Exploration

```bash
# Run for days/weeks discovering constants
ganit-a auto --config research.yaml

# Stop anytime with Ctrl+C
# Resume later from exact same point
ganit-a auto --config research.yaml  # Continues where left off!
```

### Background Discovery

```bash
# Run in background
nohup ganit-a auto --config config.yaml > discovery.log 2>&1 &

# Check progress
ganit-a checkpoint-status

# Stop when desired
pkill -INT ganit-a  # Graceful shutdown
```

### Targeted Exploration

```bash
# Explore specific parameter space exhaustively
ganit-a auto --max-iterations 1000

# Reset and start fresh
ganit-a auto --reset
```

---

## 📋 Quick Start

### 1. Start Auto-Discovery

```bash
# Simple start (uses config.yaml)
ganit-a auto

# Custom configuration
ganit-a auto --config my_config.yaml

# Limit iterations
ganit-a auto --max-iterations 100

# Reset checkpoint and start fresh
ganit-a auto --reset
```

### 2. Monitor Progress

```bash
# Check checkpoint status
ganit-a checkpoint-status

# View in web UI
# http://localhost:8000 (if API server running)
```

### 3. Stop/Resume

```bash
# Stop: Press Ctrl+C (graceful shutdown)

# Resume: Run same command again
ganit-a auto  # Automatically resumes from checkpoint!
```

---

## 🔧 How It Works

### Checkpoint System

The checkpoint tracks:

```json
{
  "version": "1.0.0",
  "status": "running",
  "total_discoveries": 42,
  "total_iterations": 156,
  "modules": {
    "series_products": {
      "explored_params": [...],  // Parameter combinations tried
      "discoveries_count": 15,
      "iterations": 50,
      "status": "active"
    },
    "continued_fractions": {
      "explored_params": [...],
      "discoveries_count": 18,
      "iterations": 52,
      "status": "active"
    },
    ...
  }
}
```

**Location**: `ganita_data/checkpoint.json`

### Discovery Cycle

Each iteration:

1. **Load checkpoint** - Check what's been explored
2. **Generate parameters** - Intelligently select next params
3. **Skip explored** - Don't repeat work
4. **Explore** - Run discovery with new parameters
5. **Store results** - Save to database
6. **Update checkpoint** - Record exploration
7. **Repeat** - Until stopped

### Graceful Shutdown

When you press `Ctrl+C`:

1. **Catch signal** - SIGINT/SIGTERM handler
2. **Stop cleanly** - Finish current iteration
3. **Save state** - Write checkpoint
4. **Export summary** - Generate report
5. **Exit** - Clean shutdown

---

## 🎮 CLI Commands

### Start Auto-Discovery

```bash
ganit-a auto [OPTIONS]

Options:
  -c, --config PATH       Configuration file (default: config.yaml)
  --db PATH               Database path (default: ganita_data/math_discovery.db)
  --artifacts PATH        Artifacts directory (default: ganita_data/artifacts)
  --checkpoint PATH       Checkpoint file (default: ganita_data/checkpoint.json)
  -n, --max-iterations N  Maximum iterations (default: unlimited)
  --reset                 Reset checkpoint and start fresh
```

### Check Status

```bash
ganit-a checkpoint-status [OPTIONS]

Options:
  --checkpoint PATH  Checkpoint file path
```

**Output**:
```
╔════════════════════════════════════╗
║       Checkpoint Status            ║
╚════════════════════════════════════╝

Status            running
Total Discoveries 42
Total Iterations  156
Last Updated      2024-01-20T15:30:45

Module Progress:
────────────────────────────────────────
Module                 Discoveries  Iterations  Explored  Status
series_products        15           50          25        active
continued_fractions    18           52          28        active
fixed_points           9            54          22        active
```

### Reset Checkpoint

```bash
ganit-a checkpoint-reset [OPTIONS]

Options:
  --checkpoint PATH  Checkpoint file path

# Prompts for confirmation
Are you sure you want to reset the checkpoint? [y/N]: y
✓ Checkpoint reset
```

---

## 🌐 Web UI Control

### Start Auto-Discovery via API

```bash
# Start API server
uvicorn ganita.api:app --reload
```

**Endpoints**:

```bash
# Start
curl -X POST http://localhost:8000/auto-discovery/start \
  -H "Content-Type: application/json" \
  -d '{
    "target_precision_digits": 600,
    "wallclock_limit_minutes": 120,
    "max_workers": 4,
    "random_seed": 42,
    "modules": []
  }'

# Stop
curl -X POST http://localhost:8000/auto-discovery/stop

# Pause
curl -X POST http://localhost:8000/auto-discovery/pause

# Resume
curl -X POST http://localhost:8000/auto-discovery/resume

# Get status
curl http://localhost:8000/auto-discovery/status
```

### Real-Time Updates

WebSocket messages:

```javascript
{
  "type": "auto_discovery_started",
  "message": "Auto-discovery started"
}

{
  "type": "auto_discovery_progress",
  "progress": {
    "total_discoveries": 42,
    "total_iterations": 156,
    ...
  },
  "session_stats": {
    "session_discoveries": 10,
    "session_iterations": 30,
    ...
  }
}

{
  "type": "auto_discovery_stopped",
  "progress": {...}
}
```

---

## ⚙️ Configuration

### Optimize for Auto-Discovery

```yaml
# config.yaml

run:
  target_precision_digits: 400  # Lower for speed
  random_seed: 42               # For reproducibility

modules:
  - name: series_products
    enabled: true
    budget_minutes: 5.0  # Shorter per-iteration budget
    parameter_grid:
      # Smaller grid for faster cycles
      template: [dirichlet, polylog]
      p: {type: range, start: 2, end: 4, step: 1}

  - name: continued_fractions
    enabled: true
    budget_minutes: 5.0
    parameter_grid:
      template: [simple, linear]
      a0: [0, 1, 2]

  - name: fixed_points
    enabled: true
    budget_minutes: 5.0
    parameter_grid:
      template: [cosine, exponential]
      scale: [0.5, 1.0]
```

### Balance Speed vs. Precision

| Mode | Precision | Budget/Module | Iterations/Hour | Use Case |
|------|-----------|---------------|-----------------|----------|
| **Fast** | 200 digits | 2 min | 10+ | Broad exploration |
| **Balanced** | 400 digits | 5 min | 5-7 | Standard discovery |
| **Deep** | 600+ digits | 10 min | 2-3 | High-confidence |

---

## 📊 Monitoring & Analytics

### Live Progress

```bash
# Terminal 1: Run auto-discovery
ganit-a auto

# Terminal 2: Watch progress
watch -n 5 ganit-a checkpoint-status
```

### Checkpoint Summary

Automatically exported to `checkpoint.summary.txt`:

```
============================================================
EXPLORATION CHECKPOINT SUMMARY
============================================================
Status: running
Created: 2024-01-20T10:00:00
Last Updated: 2024-01-20T15:30:45
Total Discoveries: 42
Total Iterations: 156
Runs: 3

Module Progress:
------------------------------------------------------------

series_products:
  Discoveries: 15
  Iterations: 50
  Explored Parameters: 25
  Status: active

continued_fractions:
  Discoveries: 18
  Iterations: 52
  Explored Parameters: 28
  Status: active

...
============================================================
```

### Database Queries

```python
from ganita.storage.db import Database

db = Database("ganita_data/math_discovery.db")

# Get all discoveries
all_discoveries = db.query_top(limit=1000)

# Filter by run
run_discoveries = db.query_top(run_id=5, limit=100)

# Get recognized constants
recognized = db.query_top(stage="recognized", limit=50)
```

---

## 🎓 Advanced Features

### Intelligent Parameter Selection

Auto-discovery doesn't just iterate through a grid—it **intelligently explores**:

1. **Grid phase** (first 10 params): Systematic grid coverage
2. **Random exploration**: Varied parameters beyond initial grid
3. **Adaptive sampling**: Focus on promising regions
4. **Deduplication**: Never explore same params twice

### State Persistence

Everything is saved:

- ✅ Parameter combinations tried
- ✅ Discoveries found
- ✅ Iteration counts
- ✅ Module status
- ✅ Timestamps
- ✅ Full audit trail

### Resource Management

- **Graceful shutdown**: No data loss
- **Memory efficient**: Streams to disk
- **CPU controlled**: Configurable workers
- **Fault tolerant**: Survives crashes

---

## 🔒 Safety & Limits

### Resource Limits

```python
# In auto_discovery.py

class AutoDiscovery:
    # Limits per iteration
    MAX_TERMS = 100000           # Per series/product
    TIMEOUT_SECONDS = 600         # Per parameter set
    MAX_MEMORY_MB = 4096          # Per worker
```

### Divergence Detection

Automatically stops if:
- Series diverges
- Radius grows instead of shrinks
- Computation takes too long
- Invalid results

### Checkpoint Safety

- Atomic writes (tmp file + rename)
- JSON validation
- Backward compatible
- Corruption detection

---

## 🐛 Troubleshooting

### Checkpoint Corrupted

```bash
# Backup current
cp ganita_data/checkpoint.json ganita_data/checkpoint.backup.json

# Reset
ganit-a checkpoint-reset

# Or manually fix JSON
vim ganita_data/checkpoint.json
```

### Stuck/Slow Progress

```bash
# Check what's running
ganit-a checkpoint-status

# Reduce precision for speed
# Edit config.yaml:
#   target_precision_digits: 200

# Increase workers
#   max_workers: 8
```

### Out of Disk Space

```bash
# Check database size
du -h ganita_data/

# Clean old traces
python -c "
from ganita.storage.artifacts import ArtifactStore
artifacts = ArtifactStore('ganita_data/artifacts')
artifacts.cleanup_old_artifacts(keep_latest=100)
"
```

---

## 📈 Performance Tips

### Maximize Throughput

1. **Lower precision** (200-400 digits)
2. **More workers** (= # CPU cores)
3. **Shorter budgets** (2-5 min/module)
4. **Smaller grids** (faster cycles)
5. **SSD storage** (faster I/O)

### Optimize for Quality

1. **Higher precision** (600+ digits)
2. **Longer budgets** (10-20 min/module)
3. **Multi-precision recognition**
4. **Cross-validation enabled**
5. **Careful parameter selection**

---

## 🎯 Best Practices

### For Long Runs

```bash
# Use screen/tmux
screen -S ganit-a
ganit-a auto
# Ctrl+A, D to detach

# Or systemd service
sudo systemctl start ganit-a-auto
```

### For Reproducibility

```yaml
# Always set random seed
run:
  random_seed: 42

# Version control your config
git add config.yaml
git commit -m "Auto-discovery config for run #5"
```

### For Collaboration

```bash
# Share checkpoint
rsync -av ganita_data/checkpoint.json colleague:/path/

# Colleague continues
ganit-a auto  # Picks up where you left off!
```

---

## 🚀 Example Session

```bash
# Day 1: Start exploration
$ ganit-a auto --config research.yaml
╔════════════════════════════════════════════╗
║  Ganit-A Auto-Discovery Mode               ║
╚════════════════════════════════════════════╝

Starting auto-discovery... (Press Ctrl+C to stop gracefully)

======================================================================
ITERATION 1
======================================================================
--- Module: series_products ---
Exploring parameters: {'template': 'dirichlet', 'p': 2, 'a': 1, 'b': 0, 'sign': 1}
  ✓ Stored discovery #1: 1.6449340668482264364724151666460251...

... (runs for hours)

^C
Stopping auto-discovery...

======================================================================
AUTO-DISCOVERY STOPPED
======================================================================
Session discoveries: 15
Total discoveries: 15
Checkpoint saved to: ganita_data/checkpoint.json


# Day 2: Resume
$ ganit-a auto --config research.yaml
Resuming from checkpoint:
  Total discoveries: 15
  Total iterations: 47

... (continues seamlessly)


# Day 7: Check progress
$ ganit-a checkpoint-status
Total Discoveries: 142
Total Iterations: 438
```

---

## 📚 See Also

- [README.md](README.md) - Main documentation
- [QUICKSTART.md](QUICKSTART.md) - Getting started
- [FEATURES.md](FEATURES.md) - Feature details
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribute

---

**Auto-Discovery Mode makes Ganit-A a truly autonomous mathematical explorer!** 🔬✨
