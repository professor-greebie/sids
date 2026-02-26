# API Stability Policy

This document describes SIDS' approach to API stability and versioning guarantees.

## Version 1.0 Stability Commitment

Starting with version 1.0.0, SIDS follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version (X.0.0): Breaking changes to stable APIs
- **MINOR** version (0.X.0): New features, backward compatible
- **PATCH** version (0.0.X): Bug fixes, backward compatible

## API Stability Levels

### Stable APIs

**Marked as**: No special annotation (default for public APIs in `src/actors.rs`)

**Guarantee**: Will not have breaking changes without a major version bump

**Stable APIs (v1.0.0+)**:

#### Core Actor System

- `start_actor_system()` - Create actor system
- `start_actor_system_with_config()` - Create actor system with custom config
- `spawn_actor()` - Spawn async actor
- `spawn_blocking_actor()` - Spawn blocking actor
- `send_message_by_id()` - Send message to actor
- `ping_actor_system()` - Verify system is responsive
- `get_actor_sender()` - Get ActorRef for an actor

#### Actor Management (v0.8.0+, stable in v1.0.0)

- `stop_actor()` - Stop specific actor
- `list_actors()` - List all actors with IDs and names
- `find_actor_by_name()` - Find actor by name
- `actor_exists()` - Check if actor exists

#### Response Handling

- `get_response_handler()` - Create response handler (recommended)
- `get_response_channel()` - Create oneshot response channel
- `get_blocking_response_channel()` - Create blocking response channel

#### Monitoring

- `get_message_count_reference()` - Get message count atomic reference
- `get_thread_count_reference()` - Get thread count atomic reference
- `get_total_messages()` - Get total messages processed
- `get_total_threads()` - Get total active threads

#### Core Types

- `ActorSystem<MType, Response>` - Main actor system type
- `ActorRef<MType, Response>` - Reference to an actor
- `Message<MType, Response>` - Message envelope
- `Actor<MType, Response>` trait - Actor behavior trait
- `ActorError` - Error type
- `ActorResult<T>` - Result type alias

### Feature-Gated Stable APIs

**Guarantee**: Stable within their feature flag, but the feature may evolve

#### Streaming Module (`feature = "streaming"`)

**Stability**: Stable for 1.0.0

- `Source<T, Mat>` - Stream source
- `Flow<In, Out, Mat>` - Stream transformation
- `Sink<T, Mat>` - Stream sink
- `StreamMessage` - Message type for streaming
- `Materializer` - Stream execution context

**Core Operations**:

- `Source::new()`, `Source::from_items()`
- `source.to_sink()` - Connect source to sink
- `source.via()` - Apply flow transformation
- `source.via_to_sink()` - Full pipeline
- `source.map()`, `source.filter()` - Functional operations

#### Supervision Module (`feature = "visualize"`)

**Stability**: Experimental (may change before 1.0.0)

**Note**: The supervision/visualization module is still under development. APIs may change
in minor versions before 1.0.0. After 1.0.0, it will follow the same stability guarantees.

### Experimental APIs

**Marked as**: `#[doc(cfg(...))]` or explicitly documented as experimental

**Guarantee**: May change in minor versions, even after 1.0.0

**Current Experimental Features**:

- Supervision visualization module (will stabilize in future release)
- Performance monitoring internals (message/thread counters may change)

## Breaking Change Policy

### What Constitutes a Breaking Change

Breaking changes require a major version bump:

- Removing public APIs
- Changing function signatures (parameters, return types)
- Changing trait definitions
- Changing behavior in ways that could break existing code
- Removing or renaming public types, structs, or enums

### What Is NOT a Breaking Change

These can happen in minor versions:

- Adding new public APIs
- Adding new optional parameters with defaults
- Adding new trait methods with default implementations
- Deprecating APIs (but not removing them)
- Bug fixes that make behavior match documented behavior
- Performance improvements
- Internal implementation changes

## Deprecation Policy

Instead of immediately removing APIs:

1. Mark as deprecated using `#[deprecated]` attribute
2. Provide alternative in deprecation message
3. Maintain for at least one minor version
4. Remove in next major version

Example:

```rust
#[deprecated(since = "1.2.0", note = "use `get_response_handler` instead")]
pub fn old_function() { ... }
```

## Feature Flag Stability

- Feature flags themselves are stable (won't be renamed/removed without major version)
- `streaming` feature: Stable in 1.0.0
- `visualize` feature: Experimental, will stabilize in future release
- Default features: Only stable features

## Minimum Supported Rust Version (MSRV)

- Current MSRV: **Rust 1.70.0**
- MSRV increases require minor version bump (not patch)
- We target stable Rust, not nightly features
- MSRV will not increase more than once per minor version

## Pre-1.0 Versions

Before 1.0.0:

- Breaking changes may occur in minor versions (0.X.0)
- We minimize breaking changes and document them in CHANGELOG
- Most core APIs are already stable (won't change before 1.0)

## Documentation Requirements

All stable APIs must have:

- Public documentation (doc comments)
- Examples in documentation
- Error conditions documented
- Test coverage

## Reporting Issues

If you encounter:

- **Undocumented breaking changes**: File a bug report
- **APIs that should be stable but aren't documented**: Open an issue
- **Experimental APIs you need stabilized**: Request stabilization

See [CONTRIBUTING.md](../CONTRIBUTING.md) for how to contribute and report issues.

## Stability Guarantees Summary

| API Category | Stability | Version Changes Allowed |
| ------------ | --------- | ----------------------- |
| Core Actor APIs | Stable | Major only |
| Actor Management APIs | Stable (v1.0+) | Major only |
| Response Handlers | Stable | Major only |
| Streaming APIs | Stable (v1.0+) | Major only |
| Supervision/Visualize | Experimental | Minor versions OK |
| Internal APIs | No guarantee | Any version |

---

**Last Updated**: December 2024 (for v1.0.0 release)

This policy ensures SIDS users can upgrade with confidence while allowing the project to evolve
and improve over time.
