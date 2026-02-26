# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Actor-Critic Example**: New `examples/actor_critic.rs` demonstrating reinforcement learning
  - Multi-armed bandit environment with 3 arms
  - Actor agent learning action policy with softmax selection
  - Critic agent learning value function estimates
  - Coordinator managing training episodes
  - Shows real-world ML pattern with coordinating actors and TD learning

## [1.0.0] - 2026-02-26

### First Stable Release 🎉

This release marks SIDS as production-ready with stable APIs and comprehensive documentation.

### Added

- **API Stability Policy**: Comprehensive stability guarantees and semantic versioning commitments
  - `docs/STABILITY.md` - Complete API stability documentation
  - Stability markers in code documentation
  - Clear distinction between stable and experimental APIs
- **Performance Benchmarks**: Comprehensive benchmark suite using Criterion
  - Actor benchmarks: spawn, messaging, concurrency, response handlers, lookup
  - Streaming benchmarks: source creation, pipelines, transformations, operations
  - Benchmark documentation in CONTRIBUTING.md
  - CI integration for benchmark validation
- **Documentation Enhancements**:
  - Module-level documentation in `lib.rs` with stability information
  - Experimental feature warnings for supervision/visualization
  - README link to stability policy

### Changed

- **API Stability Commitment**: All core APIs are now stable
  - Core actor system APIs: guaranteed stable
  - Streaming APIs: guaranteed stable
  - Supervision APIs: marked as experimental, may change in minor versions
- **CI Enhancement**: Added benchmark runs to GitHub Actions workflow
  - Benchmarks run in test mode on all PRs
  - Validates benchmarks compile without errors

### Stability Guarantees

Starting with v1.0.0, SIDS guarantees:

- **Stable APIs**: No breaking changes without major version bump
- **Semantic Versioning**: Strict adherence to semver 2.0.0
- **MSRV**: Minimum Supported Rust Version 1.70.0
- **Feature Flags**: `streaming` stable, `visualize` experimental

For complete details, see [docs/STABILITY.md](docs/STABILITY.md).

## [0.8.0] - 2026-02-26

### Added

- **Actor management APIs**: Four new core APIs for better actor control
  - `stop_actor(&mut system, id)` - Stop a specific actor by ID
  - `list_actors(&system)` - Get all actors with IDs and names
  - `find_actor_by_name(&system, name)` - Find actor ID by name
  - `actor_exists(&system, id)` - Check if actor exists
- Internal actor name tracking for all actors (not just with `visualize` feature)
- Comprehensive tests for new actor management APIs
- **CONTRIBUTING.md**: Comprehensive contributor guidelines including development setup, testing, code style, and PR process

### Removed

- **BREAKING**: Removed unused `etl` feature flag from Cargo.toml

## [0.7.0] - 2026-02-26

### Added

- **Error handling system**: Comprehensive error types (`ActorError`, `ConfigError`)
- `ActorResult<T>` and `ConfigResult<T>` type aliases for cleaner code
- `ResponseHandler` trait pattern to replace raw oneshot senders
- Proper error variants for all failure modes
- `ERROR_HANDLING.md` - Complete error handling guide
- `ERROR_HANDLING_REFACTORING.md` - Technical migration guide
- CI/CD pipeline with GitHub Actions
- Security policy (`SECURITY.md`)
- Issue templates and PR template
- Dependabot configuration
- Code coverage reporting
- Automated release workflow

### Changed

- **BREAKING**: All public APIs now return `Result` types instead of panicking
  - `send_message_by_id()` → `ActorResult<()>`
  - `send_message_to_actor()` → `ActorResult<()>`
  - `ping_system()` → `ActorResult<()>`
  - `get_actor_ref()` → `ActorResult<ActorRef<...>>`
  - `get_actor_sender()` → `ActorResult<ActorRef<...>>`
  - `Config::from_toml_str()` → `ConfigResult<Config>`
  - `Config::load_from_file()` → `ConfigResult<Config>`
- **BREAKING**: Response handlers now use `Arc<dyn ResponseHandler<Response>>` instead of `oneshot::Sender`
- Error messages are now structured with context instead of plain strings
- Updated all examples to use proper error handling
- Fixed all compiler warnings (0 warnings)

### Fixed

- Memory leak potential from abandoned oneshot channels
- Static lifetime issues with response channels
- Panic behavior in public APIs - now returns errors instead

### Migration Guide

See `ERROR_HANDLING_REFACTORING.md` for detailed migration instructions from 0.6.x.

**Quick migration:**

```rust
// Old (0.6.x)
send_message_by_id(&mut system, 0, msg).await;

// New (0.7.0)
send_message_by_id(&mut system, 0, msg).await?;
// or
send_message_by_id(&mut system, 0, msg)
    .await
    .expect("Failed to send message");
```

## [0.6.0] - 2026-01-15

### Added

- Streaming module with `Source`, `Flow`, and `Sink` actors
- Backpressure support in streaming pipelines
- File-based data sources
- Stream transformation and filtering
- Integration tests for streaming
- Visualization support with `visualize` feature flag

### Changed

- Improved actor supervision patterns
- Enhanced guardian actor capabilities
- Better concurrent message handling

### Fixed

- Race conditions in actor spawning
- Message ordering issues under high load

## [0.5.2] - 2025-12-10

### Added

- Blocking actor support with dedicated thread pools
- `spawn_blocking_actor()` function
- Response channels for blocking actors
- Examples demonstrating blocking actor patterns

### Fixed

- Thread pool exhaustion under heavy load
- Deadlock scenarios with nested actor calls

## [0.5.1] - 2025-11-20

### Fixed

- Documentation typos and broken links
- Example code compilation issues
- Missing re-exports in public API

## [0.5.0] - 2025-11-15

### Added

- Actor system with guardian actor pattern
- Message passing with `Message<MType, Response>` type
- Actor lifecycle management (spawn, stop, supervision)
- Async actor trait implementation
- Response handling with oneshot channels
- Configuration loading from TOML files
- Multiple examples (chatbot, loggers, supervision)
- Comprehensive test suite

### Changed

- Refactored actor system architecture for better performance
- Improved message routing efficiency

## [0.4.0] - 2025-10-01

### Added

- Initial actor model implementation
- Basic message passing
- Actor spawning and management

---

## Version History

- **0.7.0** (2026-02-26) - Error handling system, CI/CD, breaking API changes
- **0.6.0** (2026-01-15) - Streaming module, backpressure, supervision
- **0.5.2** (2025-12-10) - Blocking actors, thread pools
- **0.5.1** (2025-11-20) - Bug fixes, documentation
- **0.5.0** (2025-11-15) - Core actor system, configuration
- **0.4.0** (2025-10-01) - Initial release

[Unreleased]: https://github.com/professor-greebie/sids/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/professor-greebie/sids/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/professor-greebie/sids/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/professor-greebie/sids/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/professor-greebie/sids/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/professor-greebie/sids/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/professor-greebie/sids/releases/tag/v0.4.0
