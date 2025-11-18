# Contributing to Ganit-A

Thank you for your interest in contributing to Ganit-A! This document provides guidelines and instructions for contributing.

## 🎯 Ways to Contribute

- **Bug Reports**: Report bugs via GitHub Issues
- **Feature Requests**: Suggest new discovery modules or features
- **Code Contributions**: Submit pull requests for bug fixes or enhancements
- **Documentation**: Improve documentation, add examples, or write tutorials
- **Testing**: Add test cases or improve test coverage
- **Performance**: Optimize algorithms or improve scalability

## 🚀 Getting Started

### 1. Fork and Clone

```bash
git clone https://github.com/your-username/ganit-a.git
cd ganit-a
```

### 2. Set Up Development Environment

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install with development dependencies
pip install -e ".[dev]"

# Install pre-commit hooks (optional)
pip install pre-commit
pre-commit install
```

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

## 📝 Development Guidelines

### Code Style

- **Python**: Follow PEP 8
- **Formatting**: Use `black` for code formatting
- **Linting**: Use `ruff` for linting
- **Type Hints**: Add type annotations for all functions

```bash
# Format code
black src/

# Lint
ruff check src/

# Type check
mypy src/
```

### Testing

- Add tests for all new functionality
- Ensure all existing tests pass
- Aim for >80% code coverage

```bash
# Run tests
pytest tests/ -v

# With coverage
pytest tests/ --cov=ganita --cov-report=html
```

### Documentation

- Add docstrings to all public functions and classes
- Follow Google docstring format
- Update README.md if adding user-facing features

Example docstring:

```python
def discover_constant(series: Callable, precision: int) -> Discovery:
    """
    Discover a mathematical constant from a series.

    Args:
        series: Function that generates series terms
        precision: Target precision in decimal digits

    Returns:
        Discovery object with value and evidence

    Raises:
        ValueError: If series diverges or precision is invalid

    Example:
        >>> def basel(n):
        ...     return 1 / n**2
        >>> disc = discover_constant(basel, 100)
    """
```

## 🔬 Adding New Modules

To add a new discovery module:

1. Create a new file in `src/ganita/modules/`
2. Inherit from `BaseModule`
3. Implement the `explore()` method
4. Add configuration to `config.yaml`
5. Add tests in `tests/modules/`

Example:

```python
from ganita.modules.base import BaseModule
from ganita.models import Discovery

class NewModule(BaseModule):
    name = "new_module"
    description = "Description of what this module does"

    def explore(self) -> list[Discovery]:
        discoveries = []
        # Your exploration logic here
        return discoveries
```

## 🧪 Testing Your Changes

### Unit Tests

```python
# tests/modules/test_new_module.py
import pytest
from ganita.modules.new_module import NewModule

def test_new_module_basic():
    module = NewModule(evaluator, recognizer, config, env)
    discoveries = module.explore()
    assert len(discoveries) > 0
```

### Integration Tests

```python
def test_full_pipeline_with_new_module():
    config = RunConfig(modules=[
        ModuleConfig(name="new_module", enabled=True)
    ])
    run_id = run_exploration(config, db_path, artifacts_path)
    assert run_id > 0
```

## 📋 Pull Request Process

### Before Submitting

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests pass
- [ ] Documentation updated
```

### Review Process

1. Maintainers will review your PR
2. Address any requested changes
3. Once approved, your PR will be merged
4. Your contribution will be acknowledged in the release notes

## 🐛 Reporting Bugs

### Bug Report Template

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step 1
2. Step 2
3. ...

## Expected Behavior
What you expected to happen

## Actual Behavior
What actually happened

## Environment
- OS: [e.g., Ubuntu 22.04]
- Python version: [e.g., 3.11.5]
- Ganit-A version: [e.g., 1.0.0]

## Additional Context
Any other relevant information
```

## 💡 Feature Requests

### Feature Request Template

```markdown
## Feature Description
Clear description of the proposed feature

## Use Case
Why this feature would be useful

## Proposed Implementation
(Optional) Ideas for how to implement

## Alternatives Considered
Other approaches you've thought about
```

## 🎨 Areas for Contribution

### High Priority

- **New Discovery Modules**: Integrals, limits, digit sequences
- **Performance Optimizations**: Parallelization, caching
- **Recognition Bases**: Expand constant bases
- **UI Enhancements**: Mobile responsiveness, dark theme

### Medium Priority

- **Advanced Acceleration**: Additional convergence methods
- **Export Formats**: LaTeX, JSON, CSV reports
- **Benchmarking**: Comparison with other tools
- **Documentation**: Tutorials, videos, blog posts

### Good First Issues

- **Testing**: Increase test coverage
- **Documentation**: Fix typos, improve examples
- **UI**: Minor improvements, accessibility
- **Configuration**: Validation, error messages

## 📜 Code of Conduct

### Our Pledge

We pledge to make participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

**Positive behavior**:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards others

**Unacceptable behavior**:
- Trolling, insulting/derogatory comments
- Public or private harassment
- Publishing others' private information
- Other conduct which could reasonably be considered inappropriate

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Email**: ganit-a@example.com for private inquiries

## 🙏 Recognition

All contributors will be:
- Listed in CONTRIBUTORS.md
- Acknowledged in release notes
- Credited in documentation (if applicable)

Thank you for contributing to Ganit-A! 🎉
