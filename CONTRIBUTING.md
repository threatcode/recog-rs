# Contributing to Recog-RS

Thank you for your interest in contributing to Recog-RS! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70.0 or later (MSRV)
- Git
- Basic knowledge of Rust and pattern matching

### Development Setup

```bash
# Clone the repository
git clone https://github.com/threatcode/recog-rs.git
cd recog-rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check
```

## 📋 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests with all features
cargo test --all-features

# Run benchmarks (if performance-related)
cargo bench
```

### 4. Submit Pull Request

- Use descriptive title and description
- Link relevant issues
- Ensure CI passes
- Request code review

## 🏗️ Code Structure

### Core Modules

- `src/fingerprint.rs` - Core fingerprint data structures
- `src/matcher.rs` - Pattern matching engine
- `src/loader.rs` - XML fingerprint loading
- `src/error.rs` - Error handling
- `src/params.rs` - Parameter processing

### Advanced Features

- `src/async_loader.rs` - Async I/O support
- `src/plugin.rs` - Plugin architecture
- `src/analytics.rs` - Pattern analytics
- `src/wasm.rs` - WebAssembly support
- `src/ffi.rs` - C FFI bindings

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# With specific features
cargo test --features async
cargo test --all-features

# Specific test module
cargo test comprehensive_tests

# Benchmark tests
cargo bench
```

### Writing Tests

- Unit tests go in the same module
- Integration tests go in `tests/`
- Use `#[cfg(test)]` for test-only code
- Test edge cases and error conditions

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

## 📝 Code Style

### Rust Guidelines

- Follow `rustfmt` formatting
- Use `cargo clippy` with strict warnings
- Prefer `Result<T, Error>` over panics
- Document public APIs with `///`
- Use meaningful variable names

### Specific Patterns

```rust
// ✅ Good
pub fn process_fingerprints(
    db: &FingerprintDatabase,
    input: &str,
) -> RecogResult<Vec<MatchResult>> {
    // Implementation
}

// ❌ Avoid
pub fn process(db: &FingerprintDatabase, s: &str) -> Vec<MatchResult> {
    // Implementation
}
```

## 🔧 Performance Guidelines

### Benchmarking

- Use Criterion for benchmarks
- Test with realistic data sizes
- Include before/after comparisons
- Document performance implications

### Memory Safety

- Avoid unnecessary allocations
- Use iterators where possible
- Profile with `cargo profdata` if needed
- Consider streaming for large datasets

## 📚 Documentation

### API Documentation

- Document all public functions and structs
- Include examples in doc comments
- Use `#[doc]` attributes for complex documentation
- Run `cargo doc --no-deps` to verify

### README Updates

- Update feature lists
- Add new examples
- Update performance benchmarks
- Include breaking changes

## 🔄 Release Process

### Version Bumping

- Follow Semantic Versioning
- Update `Cargo.toml` version
- Update changelog
- Tag releases in Git

### Breaking Changes

- Document in CHANGELOG.md
- Update major version
- Provide migration guide
- Announce in release notes

## 🐛 Bug Reports

### Reporting Issues

- Use GitHub issue templates
- Include minimal reproduction case
- Provide environment details
- Add relevant logs/output

### Fixing Bugs

- Create issue before fixing
- Add regression tests
- Document the fix
- Consider backward compatibility

## ✨ Feature Requests

### Proposing Features

- Open issue for discussion
- Provide use case and motivation
- Consider implementation complexity
- Get community feedback

### Implementation

- Start with proof of concept
- Iterate based on feedback
- Include comprehensive tests
- Update documentation

## 🤝 Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Follow Rust community standards

### Communication

- Use GitHub issues for bugs/features
- Use discussions for questions
- Be patient with responses
- Help others when possible

## 📊 Project Areas

### High Priority

- Core fingerprint matching
- XML parsing performance
- Memory efficiency
- Error handling

### Medium Priority

- Async I/O improvements
- Plugin system enhancements
- Documentation improvements
- Additional matchers

### Low Priority

- Additional language bindings
- GUI tools
- Advanced analytics
- Experimental features

## 🏆 Recognition

Contributors are recognized in:

- README.md contributors section
- Release notes
- Git commit history
- GitHub contributor statistics

## 📞 Getting Help

- **Issues**: [GitHub Issues](https://github.com/threatcode/recog-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/threatcode/recog-rs/discussions)
- **Documentation**: [docs.rs/recog](https://docs.rs/recog)

---

Thank you for contributing to Recog-RS! Your contributions help make fingerprint recognition faster, safer, and more accessible to everyone.
