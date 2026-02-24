# SIDS - An Actor Model Approach to Data Collection in RUST

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

## What This Does

This project demonstrates a practical approach to building concurrent systems in Rust using:

- **Actor Model**: A message-passing architecture with isolated, concurrent actors
- **Streaming Pipelines**: Functional reactive programming patterns for data processing

The project allows for abstraction between Tokio-based asynchronous actors and blocking actors, providing a flexible foundation for concurrent application development.

### Basic Concepts

An actor implements an `Actor<MType, Response>` trait that includes a `receive` function accepting a message type of `Message<MType, Response>`.

The `Message` struct covers the most common Actor behaviors (stop, responses etc.), but you can add more as part of the payload, which is of type MType.

MType can be any base type (`String`, `u32` etc.) or an enum provided that it has Send features and can have static lifetime. Enums are powerful in Rust, so they are highly recommended. See the [Rust documentation on enum types for more information](https://doc.rust-lang.org/book/ch06-00-enums.html)

`Response` is any enum that actors use to send return messages back to the sender. A generic `ResponseMessage` can be used by default.

Once you choose an `MType`, the `ActorSystem` uses the same message type throughout the system. Currently, only one `MType` is allowed; however, with Rust's enums, there is significant capacity for variance in message types.

```rust
let mut actor_system = sids::actors::start_actor_system::<MType, Response>();
```

Starting an actor system initializes the system and runs a 'boss' actor called the `Guardian` with an id of 0. You can ping the boss using `sids::actors::ping_actor_system(&actor_system);`

You can add an actor to the system by creating a structure that implements the `Actor<MType>` trait. All actors must receive a `Message<MType>`.

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
async fn main() -> Result<()> {
    let my_actor = MyActor;

    let mut actor_system = sids::actors::start_actor_system::<MyMessage, ResponseMessage>();
    // gets a oneshot channel to receive a response from the system.
    let (tx, rx) = sids::actors::get_response_channel(&actor_system);
    let message = Message {
        payload: Some(MyMessage::HELLO),
        stop: false,
        responder: Some(tx),
        blocking: None,
    };
    spawn_actor(&mut actor_system, my_actor, Some("My Actor".to_string())).await;
    // guardian is 0, so our actor id will be #1.
    send_message_by_id(&mut actor_system, 1, message).await;
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

For more details, see [COVERAGE.md](COVERAGE.md).

## The Future

From a prototype perspective, this is final version of this project, except for performance and safety tweaks.

We will also include some more advanced examples, including using the Actor System to do Actor-Critic Machine Learning work.

## Citations

The following resources helped me a lot during the building of this demonstration.

- Mara Bos (2023) *Rust Atomics and Locks: Low-level concurrency in Practice.* O'Reilly Press.
- Alice Ryhl (2021) [*Actors with Tokio*](https://ryhl.io/blog/actors-with-tokio/) [Blog Post].
