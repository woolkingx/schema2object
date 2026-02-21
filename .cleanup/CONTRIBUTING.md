# Contributing to schema2object

Thanks for your interest in contributing! This project follows a **low-maintenance philosophy** to keep the codebase stable and focused.

## Maintenance Status

🟡 **Maintenance Mode**: This project is feature-complete for its intended use case (JSON Schema Draft-07 with dot-access). We accept:
- ✅ Bug fixes
- ✅ Documentation improvements
- ✅ Test coverage improvements
- ✅ Draft-07 compliance fixes
- ⚠️ New features (rarely, must have strong justification)

## How to Contribute

### Reporting Bugs

1. Check [existing issues](../../issues) first
2. Include a minimal reproducible example
3. Specify your Python version and OS

### Submitting Pull Requests

**Before you start:**
1. Open an issue first to discuss the change
2. For small fixes, a PR without issue is fine

**PR Requirements:**
- [ ] Tests pass: `python -m pytest tests/test_objecttree.py -v`
- [ ] New code has tests (if applicable)
- [ ] Documentation updated (if behavior changes)
- [ ] No new dependencies added
- [ ] Code follows existing style (no formatters required)

**PR Process:**
```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/schema2object.git

# 2. Create branch
git checkout -b fix-issue-123

# 3. Make changes and test
cd /path/to/parent
python -m pytest tests/test_objecttree.py -v

# 4. Commit and push
git add .
git commit -m "Fix: description of fix"
git push origin fix-issue-123

# 5. Open PR on GitHub
```

## Development Setup

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/schema2object.git
cd schema2object

# No dependencies needed! Just stdlib
python -m pytest ../tests/test_objecttree.py -v
```

## Code Style

- Follow existing code style (no strict enforcement)
- Keep functions focused and simple
- Add docstrings for public APIs
- Comment complex logic

## Testing Guidelines

- All public APIs must have tests
- Test both happy path and error cases
- Keep tests readable and focused
- Test naming: `test_<what>_<scenario>`

Example:
```python
def test_type_validation_rejects_wrong_type():
    schema = {'properties': {'age': {'type': 'integer'}}}
    obj = ObjectTree({}, schema=schema)
    with pytest.raises(TypeError):
        obj.age = 'not_an_int'
```

## What We Won't Accept

❌ New dependencies (keep it stdlib-only)
❌ Breaking API changes (without major version bump)
❌ Features outside Draft-07 scope
❌ Over-engineered solutions
❌ Code style refactors (without functional improvement)

## Questions?

Open an issue with the `question` label.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
