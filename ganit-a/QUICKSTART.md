# Ganit-A Quick Start Guide

Get started with mathematical discovery in 5 minutes!

## ⚡ 60-Second Setup

```bash
# 1. Run setup script
chmod +x setup.sh
./setup.sh

# 2. Activate environment
source venv/bin/activate

# 3. Run demo
python examples/demo.py
```

Done! You've just discovered mathematical constants! 🎉

---

## 🎯 5-Minute Tutorial

### Step 1: Run Your First Discovery

```bash
ganit-a run --config config.yaml
```

This starts a discovery run that will:
- Explore series, products, continued fractions, and fixed points
- Evaluate candidates with rigorous interval arithmetic
- Recognize values using PSLQ against known constants
- Store results in SQLite database

**Time**: ~5-30 minutes (configurable)

### Step 2: View Your Discoveries

```bash
# Show top 10 discoveries
ganit-a top

# Filter by recognized constants
ganit-a top --stage recognized

# Show details of discovery #5
ganit-a show 5
```

**Output**:
```
╔═══════════════════════════════════════════╗
║         Top 10 Discoveries                ║
╚═══════════════════════════════════════════╝

ID  Value                          Stage        Score
─────────────────────────────────────────────────────
1   1.64493406684822643647...      recognized   0.8234
2   0.69314718055994530941...      recognized   0.7891
3   0.78539816339744830961...      recognized   0.7456
...
```

### Step 3: Launch the Web UI

```bash
# Terminal 1: Start API server
uvicorn ganita.api:app --reload

# Terminal 2: Open web interface
# Simply open web/index.html in your browser
```

Navigate to `http://localhost:8000` and explore:
- 📊 Dashboard with statistics
- 🔍 Discovery browser
- 📈 Interactive convergence plots
- ⚙️ Configuration panel

---

## 🎓 Learn by Example

### Example 1: Evaluate a Series

```python
from ganita.services.evaluator import Evaluator
import mpmath as mp

# Create evaluator
eval = Evaluator({
    "precision_ramp": [128, 256],
    "stop_radius": "1e-100",
    "max_terms": 10000
})

# Define Basel series: Σ 1/n²
def basel(n):
    return mp.mpf(1) / mp.mpf(n**2)

# Evaluate
result = eval.series_limit(basel, {}, alternating=False)

print(f"Value: {result.final.center}")
print(f"π²/6:  {mp.pi**2 / 6}")
# They match!
```

### Example 2: Recognize a Constant

```python
from ganita.services.recognizer import Recognizer

recognizer = Recognizer({
    "bases": [{
        "name": "small",
        "symbols": ["1", "PI", "LOG2"],
        "max_height": 1000
    }],
    "pslq_precision_digits": [200]
})

# Try to recognize π/4
value = str(mp.pi / 4)
result = recognizer.recognize(value)

if result.success:
    print(f"Relation: {result.relation_tex}")
    # Output: VALUE = \frac{1}{4} \pi
```

### Example 3: Find a Fixed Point

```python
# Find x where x = cos(x) (Dottie number)
result = eval.fixed_point(
    f=lambda x: mp.cos(x),
    initial=mp.mpf(0.5),
    params={}
)

print(f"Dottie number: {result.final.center}")
# ≈ 0.739085133215...
```

---

## 🎨 Customize Your Run

Edit `config.yaml`:

```yaml
run:
  target_precision_digits: 600     # Higher = more accurate
  wallclock_limit_minutes: 120     # How long to run
  random_seed: 42                  # For reproducibility

modules:
  - name: series_products
    enabled: true
    budget_minutes: 40.0
    parameter_grid:
      template: [dirichlet, polylog]
      p: {type: range, start: 2, end: 6, step: 1}
```

Then run:

```bash
ganit-a run --config my_config.yaml
```

---

## 📊 Understanding Output

### Confidence Stages

| Stage | Meaning |
|-------|---------|
| `observed` | Value computed but not certified |
| `certified` | Interval bounds rigorously proven |
| `recognized` | Matched to known constant (e.g., π/6) |
| `cross_validated` | Verified by multiple methods |

### Rank Score

Score from 0-1 based on:
- **Precision**: How accurate
- **Recognition**: How simple
- **Validation**: Cross-method agreement
- **Convergence**: How smooth
- **Efficiency**: Computational cost

Higher = more interesting/trustworthy

---

## 🔧 Troubleshooting

### "Command not found: ganit-a"

```bash
# Ensure virtual environment is activated
source venv/bin/activate

# Reinstall
pip install -e .
```

### "Database locked"

```bash
# Another process is using the database
# Wait or specify different database:
ganit-a run --db my_discoveries.db
```

### "Module import error"

```bash
# Install dependencies
pip install -e .

# Or with extras:
pip install -e ".[dev]"
```

### Web UI not loading

```bash
# Check server is running
curl http://localhost:8000/health

# Should return: {"status": "healthy", ...}
```

---

## 🚀 Next Steps

### Learn More

- Read [README.md](README.md) for full documentation
- See [FEATURES.md](FEATURES.md) for advanced capabilities
- Check [CONTRIBUTING.md](CONTRIBUTING.md) to contribute

### Try Advanced Features

```bash
# Run with custom precision
ganit-a run --config high_precision.yaml

# Export discoveries
ganit-a export --run-id 1 --format json

# Compare runs
ganit-a compare --run-ids 1,2,3
```

### Explore Modules

- **Series & Products**: Explore Dirichlet series, polylogs, Euler products
- **Continued Fractions**: Simple, linear, quadratic CFs
- **Fixed Points**: Cosine, exponential, nested radicals
- **Custom Modules**: Write your own discovery algorithms!

### Use Programmatically

```python
from ganita.orchestrator import Orchestrator
from ganita.models import RunConfig

config = RunConfig(...)
orchestrator = Orchestrator(config, "my.db", "artifacts/")
run_id = orchestrator.run()

# Access results
top_discs = orchestrator.get_top_discoveries(limit=10)
```

---

## 💡 Tips & Tricks

1. **Start Small**: Use lower precision (200-300 digits) for faster exploration
2. **Use Web UI**: Visual feedback makes exploration more engaging
3. **Check Logs**: `ganita_run.log` has detailed progress
4. **Save Configs**: Create config presets for different use cases
5. **Cross-Validate**: Higher confidence in cross-validated discoveries
6. **Parameter Grids**: Systematic vs. random exploration trade-offs
7. **Monitor Resources**: Watch memory/CPU with long runs

---

## 🎯 Common Use Cases

### Research

```bash
# Exhaustive high-precision search
ganit-a run --config research_grade.yaml
```

### Education

```python
# Interactive demonstration
from examples.demo import demo_evaluator
demo_evaluator()
```

### Benchmarking

```bash
# Test different acceleration methods
ganit-a benchmark --methods wynn,shanks,levin
```

### Discovery

```bash
# Novel constant hunting
ganit-a run --config novel_search.yaml --stage novel_candidate
```

---

## 📞 Get Help

- **Documentation**: [README.md](README.md)
- **Issues**: [GitHub Issues](https://github.com/your-org/ganit-a/issues)
- **Questions**: [GitHub Discussions](https://github.com/your-org/ganit-a/discussions)
- **Email**: ganit-a@example.com

---

## 🎉 You're Ready!

You now know enough to start discovering mathematical constants!

```bash
# Start exploring!
ganit-a run

# Happy discovering! 🔬✨
```

---

[⬅️ Back to README](README.md) | [📚 Full Documentation](README.md#documentation) | [🎨 Features](FEATURES.md)
