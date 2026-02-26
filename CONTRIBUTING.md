# Contributing to SIDS

Thank you for your interest in contributing to SIDS!
This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Benchmarks](#benchmarks)
- [Code Style](#code-style)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Review Process](#review-process)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## Ways to Contribute

### Reporting Bugs

Before creating a bug report:

- Check the [existing issues](https://github.com/professor-greebie/sids/issues) to avoid duplicates
- Collect relevant information (OS, Rust version, SIDS version, error messages)

Create a bug report using the bug report template with:

- Clear, descriptive title
- Steps to reproduce
- Expected vs actual behavior
- Code samples if applicable
- Environment details

### Suggesting Features

Feature requests are welcome! Please:

- Check existing feature requests first
- Use the feature request template
- Clearly describe the use case and benefits
- Consider backwards compatibility

### Contributing Code

We welcome pull requests for:

- Bug fixes
- New features (discuss in an issue first for large changes)
- Documentation improvements
- Test coverage improvements
- Performance optimizations

### Improving Documentation

Documentation improvements are highly valued:

- Fix typos or clarify existing docs
- Add examples
- Improve API documentation
- Update guides for new features

## Development Setup

### Prerequisites

- Rust 1.70+ (check with `rustc --version`)
- Git
- A code editor (VS Code with rust-analyzer recommended)

### Clone and Build

```bash
git clone https://github.com/professor-greebie/sids.git
cd sids

# Build the project
cargo build

# Run tests
cargo test --lib --features streaming

# Run examples
cargo run --example loggers
```

### Project Structure

```text
sids/
├── src/
│   ├── actors/          # Core actor system
│   ├── streaming/       # Streaming module (feature: streaming)
│   ├── supervision.rs   # Supervision system (feature: visualize)
│   ├── config.rs        # Configuration
│   └── lib.rs          # Public API
├── examples/           # Example programs
├── architecture/       # Design documentation
└── docs/              # Additional documentation
```

## Making Changes

### Branching Strategy

1. Fork the repository
2. Create a feature branch from `main`:

   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-number-description
   ```

### Feature Flags

SIDS uses feature flags for optional functionality:

- `streaming` - Streaming data processing
- `visualize` - Actor supervision and visualization

When adding code:

- Use `#[cfg(feature = "feature-name")]` for feature-gated code
- Test with and without features enabled
- Update documentation to mention feature requirements

## Testing

### Running Tests

```bash
# All tests with all features
cargo test --lib --features streaming

# Specific module tests
cargo test --lib actors::tests
cargo test --lib streaming::tests

# Doc tests
cargo test --doc

# Integration tests (if applicable)
cargo test --test '*'
```

### Writing Tests

- **Unit tests**: Place in the same file as the code in a `mod tests` block
- **Integration tests**: Place in `tests/` directory
- **Doc tests**: Add examples to function documentation

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async code
    }
}
```

### Coverage

We use `cargo-llvm-cov` for coverage:

```bash
# Run coverage
.\run-coverage.ps1  # Windows
./run-coverage.sh   # Unix
```

Aim for:

- New code: 80%+ coverage
- Critical paths: 90%+ coverage
- Bug fixes: Add regression tests

## Benchmarks

SIDS includes performance benchmarks to track performance over time and catch regressions.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench actor_benchmarks
cargo bench --bench streaming_benchmarks --features streaming

# Run benchmarks in test mode (quick validation)
cargo bench --bench actor_benchmarks -- --test
```

### Benchmark Suites

#### Actor Benchmarks (`benches/actor_benchmarks.rs`)

Measures core actor system performance:

- **Actor spawning**: Single and bulk actor creation (10-500 actors)
- **Message passing**: Message send latency and throughput (100-1000 messages)
- **Concurrent messaging**: Multiple actors communicating simultaneously (5-20 actors)
- **Response handlers**: Round-trip message-response patterns (10-100 messages)
- **Actor lookup**: Finding actors by name in systems of varying sizes (10-100 actors)

#### Streaming Benchmarks (`benches/streaming_benchmarks.rs`)

Measures streaming pipeline performance (requires `--features streaming`):

- **Source creation**: Creating sources from single items and collections (10-1000 items)
- **Source-to-sink**: End-to-end pipeline latency and throughput (10-100 items)
- **Flow transformations**: Data transformation performance via flows (10-100 items)
- **Map/filter operations**: Functional transformation overhead

### Interpreting Results

Criterion produces detailed HTML reports in `target/criterion/`:

```bash
# View reports (open in browser)
target/criterion/report/index.html
```

Key metrics:

- **Time per operation**: Lower is better
- **Throughput**: Higher is better (operations/second)
- **Change from baseline**: Shows performance regression/improvement
- **Outliers**: Identify inconsistent performance

### Performance Expectations

Approximate baseline performance on typical hardware:

| Operation | Expected Performance |
| --------- | -------------------- |
| Actor spawn | ~5-20 μs per actor |
| Message send (no response) | ~2-10 μs |
| Message send (with response) | ~10-50 μs |
| Concurrent messaging (10 actors) | ~50-200 μs |
| Source-to-sink (100 items) | ~5-20 ms |
| Flow transformation | Similar to source-to-sink |

**Note**: Performance varies significantly by:

- CPU speed and core count
- System load
- Tokio runtime configuration
- Message size and complexity

### Adding New Benchmarks

When adding performance-sensitive features:

1. Add benchmarks to appropriate suite or create new suite
2. Follow existing patterns (use `criterion::black_box` for inputs/outputs)
3. Use meaningful benchmark names and groups
4. Test with multiple input sizes for throughput benchmarks
5. Document what the benchmark measures

Example benchmark:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            let input = black_box(setup_input());
            my_operation(input)
        });
    });
}

criterion_group!(benches, bench_my_feature);
criterion_main!(benches);
```

For async operations, use `b.to_async(&runtime)`:

```rust
use tokio::runtime::Runtime;

fn bench_async_feature(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("async_operation", |b| {
        b.to_async(&rt).iter(|| async {
            let input = black_box(setup_input());
            async_operation(input).await
        });
    });
}
```

## Code Style

### Formatting

We use `rustfmt` with default settings:

```bash
cargo fmt --all
```

### Linting

We use `clippy` with warnings denied in CI:

```bash
cargo clippy --features streaming -- -D warnings
```

Fix all clippy warnings before submitting.

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `Result` types instead of panicking (exceptions: examples and tests)
- Document public APIs with doc comments
- Add examples to function documentation when helpful
- Use meaningful variable and function names
- Keep functions focused and concise

### Error Handling

Since v0.7.0, we use proper error types:

```rust
// Good
pub fn my_function() -> ActorResult<T> {
    something_that_can_fail()?;
    Ok(result)
}

// Avoid (except in examples/tests)
pub fn my_function() -> T {
    something_that_can_fail().expect("error message")
}
```

### Documentation

- Use `///` for public item documentation
- Use `//!` for module-level documentation
- Include examples in doc comments for non-trivial functions
- Document panics, errors, and safety concerns

Example:

```rust
/// Creates a new actor system with default configuration.
///
/// The guardian actor is automatically spawned with ID 0.
///
/// # Examples
///
/// ```rust
/// use sids::actors;
///
/// let actor_system = actors::start_actor_system::<String, ResponseMessage>();
/// ```
pub fn start_actor_system<MType, Response>() -> ActorSystem<MType, Response>
```

## Commit Guidelines

### Commit Message Format

```text
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting, no code change
- `refactor`: Code restructuring, no behavior change
- `perf`: Performance improvement
- `test`: Adding/updating tests
- `chore`: Build, CI, dependencies

**Examples:**

```text
feat(actors): add stop_actor and list_actors APIs

Add four new core APIs for better actor management:
- stop_actor: Stop individual actors
- list_actors: Get all actors with IDs and names
- find_actor_by_name: Find actor ID by name
- actor_exists: Check if actor exists

Closes #42
```

```text
fix(streaming): handle empty source edge case

Fix panic when Source is created with empty iterator.
Add regression test.

Fixes #38
```

### Breaking Changes

Mark breaking changes in commit messages:

```text
feat(actors)!: change Message struct to use ResponseHandler

BREAKING CHANGE: Message.responder now uses BoxedResponseHandler
instead of tokio::sync::oneshot::Sender. Update code to use
get_response_handler() function instead of oneshot::channel().

Migration guide in GUIDE.md.
```

## Pull Request Process

### Before Submitting

- [ ] Code compiles without errors
- [ ] All tests pass (`cargo test --lib --features streaming`)
- [ ] No clippy warnings (`cargo clippy --features streaming -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Benchmarks pass if applicable (`cargo bench -- --test`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for user-facing changes)
- [ ] Examples work if applicable

### Submitting PR

1. **Push your branch** to your fork
2. **Open a pull request** against `main`
3. **Fill out the PR template** completely
4. **Link related issues** (e.g., "Fixes #42")
5. **Wait for CI** to pass
6. **Respond to review** feedback

### PR Title Format

Follow the commit message format:

```text
feat(actors): add actor management APIs
fix(streaming): handle empty source edge case
docs: improve ResponseHandler examples
```

## Review Process

### What We Look For

- **Correctness**: Does it work as intended?
- **Tests**: Are there adequate tests?
- **Documentation**: Is the API documented?
- **Code quality**: Is it readable and maintainable?
- **Performance**: Are there performance concerns?
- **Breaking changes**: Are they necessary and documented?

### Timeline

- Initial review: Within 3-5 days
- Follow-up reviews: Within 1-2 days
- Merge: After approval and CI passes

### After Approval

Once approved and CI passes, a maintainer will merge your PR. Thank you for contributing!

## Questions?

- **GitHub Discussions**: For questions and general discussion
- **Issues**: For bug reports and feature requests
- **Email**: <rdeschamps@conestogac.on.ca> (for security issues, see SECURITY.md)

## License

By contributing to SIDS, you agree that your contributions will be licensed under the Apache-2.0 License.

---

Thank you for making SIDS better!
