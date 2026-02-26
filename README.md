# SIDS - An Actor Model Approach to Data Collection in RUST

[![CI](https://github.com/professor-greebie/sids/actions/workflows/ci.yml/badge.svg)](https://github.com/professor-greebie/sids/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/sids.svg)](https://crates.io/crates/sids)
[![Documentation](https://docs.rs/sids/badge.svg)](https://docs.rs/sids)
[![License](https://img.shields.io/crates/l/sids.svg)](LICENSE.md)

This is an experimental actor-model system library built in Rust. The repository has a few Mermaid diagrams
and examples available for you to examine if you are interested in implementing the approach yourself.

## Getting Started

Run the example logging demonstration:

```bash
git clone https://github.com/professor-greebie/sids
cd sids
cargo run --example loggers
```

For a streaming example, run:

```bash
cargo run --example source --features streaming
```

For an actor-critic machine learning example:

```bash
cargo run --example actor_critic
```

## What This Does

This project demonstrates a practical approach to building concurrent systems in Rust using:

- **Actor Model**: A message-passing architecture with isolated, concurrent actors
- **Streaming Pipelines**: Functional reactive programming patterns for data processing

The project allows for abstraction between Tokio-based asynchronous actors and blocking actors,
providing a flexible foundation for concurrent application development.

### Basic Concepts

An actor implements an `Actor<MType, Response>` trait that includes a `receive` function accepting a
message type of `Message<MType, Response>`.

The `Message` struct covers the most common Actor behaviors (stop, responses etc.), but you can add
more as part of the payload, which is of type MType.

MType can be any base type (`String`, `u32` etc.) or an enum provided that it has Send features and
can have static lifetime. Enums are powerful in Rust, so they are highly recommended. See the
[Rust documentation on enum types for more information](https://doc.rust-lang.org/book/ch06-00-enums.html)

`Response` is any enum that actors use to send return messages back to the sender. A generic
`ResponseMessage` can be used by default.

Once you choose an `MType`, the `ActorSystem` uses the same message type throughout the system.
Currently, only one `MType` is allowed; however, with Rust's enums, there is significant capacity
for variance in message types.

```rust
let mut actor_system = sids::actors::start_actor_system::<MType, Response>();
```

Starting an actor system initializes the system and runs a 'boss' actor called the `Guardian` with
an id of 0. You can ping the boss using `sids::actors::ping_actor_system(&actor_system);`

You can add an actor to the system by creating a structure that implements the `Actor<MType>` trait.
All actors must receive a `Message<MType>`.

```rust

use sids::actors::actor::Actor;
use sids::actors::messages::{Message, ResponseMessage};
use log::info;

#[derive(Debug, Clone)]
enum MyMessage {
    Hello,
    Goodbye,
    Ghost,
}

// you can include some attributes like a name if you wish
struct MyActor;
impl Actor<MyMessage, ResponseMessage> for MyActor {
    async fn receive(&mut self, message: Message<MyMessage, ResponseMessage>) {
        if let Message {
            payload,
            stop,
            responder,
            blocking,
        } = message {
            if let Some(msg) = payload {
                info!("Message received {:?}", msg);
            }
            if let Some(respond) = responder {
                respond.send(ResponseMessage::Success).ok();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_actor = MyActor;

    let mut actor_system = sids::actors::start_actor_system::<MyMessage, ResponseMessage>();
    
    // Get a response handler for receiving the response
    let (handler, rx) = sids::actors::get_response_handler::<ResponseMessage>();
    
    let message = Message {
        payload: Some(MyMessage::Hello),
        stop: false,
        responder: Some(handler),
        blocking: None,
    };
    
    spawn_actor(&mut actor_system, my_actor, Some("My Actor".to_string())).await;
    
    // Guardian is 0, so our actor id will be #1
    send_message_by_id(&mut actor_system, 1, message).await?;
    
    if let Ok(response) = rx.await {
        info!("Response received from actor {:?}", response);
    }

    Ok(())
}

## Streaming Module

The streaming module provides a functional reactive programming (FRP) approach to data processing built on top of the actor system. It allows you to create pipelines that process data through various transformations in a non-blocking, efficient manner.

### Key Components

**Source**: Entry point for data into the pipeline. Generates or reads data and emits it downstream.

**Flow**: Transforms messages as they pass through. Can modify, filter, or enrich data in the pipeline.

**Sink**: Terminal point in a pipeline that consumes messages and performs side effects (e.g., printing, writing to a file, storing in a database).

**Materializer**: Executes the pipeline by connecting sources, flows, and sinks within the actor system.

### Streaming Example

```bash
# Run the streaming example with the feature enabled
cargo run --example source --features streaming
```

This example demonstrates:

```rust
use sids::streaming::{Source, Flow, Sink, StreamMessage, NotUsed};

#[tokio::main]
async fn main() {
    let mut actor_system = sids::actors::start_actor_system();

    // Create a source that emits a string
    let source = Source::new("hello world".to_string(), NotUsed);

    // Create a flow that transforms the message
    let flow = Flow::new("UppercaseFlow".to_string(), |msg: StreamMessage| {
        match msg {
            StreamMessage::Text(text) => StreamMessage::Text(text.to_uppercase()),
            other => other,
        }
    });

    // Create a sink that consumes the message
    let sink = Sink::new("PrintSink".to_string(), |msg: StreamMessage| {
        match msg {
            StreamMessage::Text(text) => println!("Result: {}", text),
            StreamMessage::Complete => println!("Stream finished!"),
            _ => {}
        }
    });

    // Connect the pipeline: source -> flow -> sink
    let _materializer = source
        .add_flow(&mut actor_system, flow)
        .await
        .to_sink(&mut actor_system, sink)
        .await;
}
```

### Building with Streaming

The streaming module is an optional feature. To build and test with streaming enabled:

```bash
# Build with streaming
cargo build --features streaming

# Run tests with streaming
cargo test --features streaming --lib
```

### Actor-Critic Machine Learning Example

```bash
# Run the actor-critic reinforcement learning example
cargo run --example actor_critic
```

This example demonstrates a practical reinforcement learning system using the actor-critic pattern:

- **Environment**: Multi-armed bandit with 3 arms (different reward probabilities)
- **Actor Agent**: Learns which arm to pull (action policy)
- **Critic Agent**: Evaluates expected rewards (value function)
- **Coordinator**: Manages the training loop with message passing

The example shows:

- Multiple coordinating actors working together
- Request-response patterns using `get_response_handler`
- Temporal difference (TD) learning with actor-critic updates
- Real-world machine learning patterns in concurrent systems

After 500 episodes, the actor learns to prefer the arm with the highest reward probability (50%),
demonstrating how SIDS can be used for machine learning applications where actors coordinate to
learn optimal policies.

## Testing and Coverage

The project includes comprehensive test coverage across all modules:

- **Streaming module**: 36 tests
- **Actor module**: 30 tests
- **Actor System module**: 19 tests
- **Total**: 85 tests

### Running Tests

```bash
# Run all tests with streaming feature
cargo test --features streaming --lib

# Run specific module tests
cargo test --features streaming --lib streaming::tests
cargo test --features streaming --lib actors::tests
cargo test --features streaming --lib actors::actor::tests
cargo test --features streaming --lib actors::actor_system::tests
```

## Configuration

This library can be configured using a TOML file. Keep your real config out of git and copy the example:

```bash
copy sids.config.example.toml sids.config.toml
```

Example snippet:

```toml
[actor_system]
actor_buffer_size = 100
shutdown_timeout_ms = 5000
```

Usage in code:

```rust
use sids::config::SidsConfig;

let config = SidsConfig::load_from_file("sids.config.toml")
    .expect("Failed to load config");
let mut actor_system = sids::actors::start_actor_system_with_config(config);
```

### Code Coverage

The project uses `cargo-llvm-cov` for code coverage analysis, configured to comply with institutional file system constraints.

**Quick start:**

```powershell
.\run-coverage.ps1
```

This will automatically:

- Install `cargo-llvm-cov` if needed
- Run tests with coverage instrumentation
- Generate an HTML coverage report
- Open the report in your browser
- Keep all artifacts within the project directory

For more details, see [docs/COVERAGE.md](docs/COVERAGE.md).

## Documentation

- **[GUIDE.md](GUIDE.md)** - Complete user guide covering error handling, response handling, supervision,
  and shutdown
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and migration guides
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines for developers
- **[docs/STABILITY.md](docs/STABILITY.md)** - API stability policy and semantic versioning guarantees
- **[docs/architecture/](architecture/)** - System architecture and design documentation
- **[API Documentation](https://docs.rs/sids)** - Complete API reference on docs.rs

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Development setup
- Testing guidelines
- Code style requirements
- Pull request process

## The Future

From a prototype perspective, this is the final version of this project, except for performance and
safety tweaks.

The project includes advanced examples demonstrating real-world use cases, including actor-critic
machine learning with reinforcement learning agents (see `examples/actor_critic.rs`).

## Citations

The following resources helped me a lot during the building of this demonstration.

- Mara Bos (2023) *Rust Atomics and Locks: Low-level concurrency in Practice.* O'Reilly Press.
- Alice Ryhl (2021) [*Actors with Tokio*](https://ryhl.io/blog/actors-with-tokio/) [Blog Post].
