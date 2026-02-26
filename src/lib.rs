//! # SIDS - Simple Implementation of a Distributed System
//!
//! SIDS is an actor-based framework for building concurrent applications in Rust.
//! It provides an ergonomic API for actor systems, message passing, and streaming data processing.
//!
//! ## Features
//!
//! - **Core Actor System**: Stable APIs for actor management and message passing
//! - **Streaming**: Functional reactive streams with source/flow/sink patterns (requires `streaming` feature)
//! - **Supervision**: Actor supervision and visualization (requires `visualize` feature, **experimental**)
//!
//! ## API Stability
//!
//! SIDS follows [Semantic Versioning](https://semver.org/). Starting with v1.0.0:
//!
//! - **Stable APIs**: Core actor system and streaming module
//! - **Experimental APIs**: Supervision/visualization module (may change in minor versions)
//!
//! See [docs/STABILITY.md](../docs/STABILITY.md) for detailed stability guarantees.
//!
//! ## Quick Start
//!
//! ```rust
//! use sids::actors::{self, actor::Actor, messages::Message};
//!
//! # struct MyActor;
//! # impl Actor<String, sids::actors::messages::ResponseMessage> for MyActor {
//! #     async fn receive(&mut self, message: Message<String, sids::actors::messages::ResponseMessage>) {
//! #         // Handle message
//! #     }
//! # }
//! #
//! # async fn example() {
//! // Create an actor system
//! let mut system = actors::start_actor_system::<String, actors::messages::ResponseMessage>();
//!
//! // Spawn an actor
//! actors::spawn_actor(&mut system, MyActor, Some("my-actor".to_string())).await;
//! # }
//! ```
//!
//! For detailed guides, see [GUIDE.md](../GUIDE.md) and [README.md](../README.md).

pub mod actors;
pub mod config;

/// Streaming data processing module with reactive streams.
///
/// **Stability**: Stable as of v1.0.0
///
/// Provides Source, Flow, and Sink abstractions for building streaming data pipelines.
/// Requires the `streaming` feature flag.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "streaming")]
/// # {
/// use sids::streaming::source::Source;
/// use sids::streaming::stream_message::NotUsed;
///
/// let source = Source::new("data".to_string(), NotUsed);
/// let transformed = source.map(|s| s.to_uppercase());
/// # }
/// ```
#[cfg(feature = "streaming")]
pub mod streaming;

/// Actor supervision and visualization module.
///
/// **Stability**: ⚠️ **EXPERIMENTAL** - APIs may change in minor versions before stabilization
///
/// This module provides tools for supervising actors and visualizing actor system behavior.
/// Requires the `visualize` feature flag.
///
/// The supervision APIs are still evolving and may have breaking changes in minor versions
/// until they are stabilized in a future release. See [docs/STABILITY.md](../docs/STABILITY.md)
/// for more information.
#[cfg(feature = "visualize")]
pub mod supervision;

/// Supervision export functionality.
///
/// **Stability**: ⚠️ **EXPERIMENTAL** - Part of the supervision module
#[cfg(feature = "visualize")]
pub mod supervision_export;
#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        assert_eq!(2 + 2, 4);
    }
}
