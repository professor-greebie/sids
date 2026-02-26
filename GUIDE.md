# SIDS User Guide

Complete guide for building concurrent applications with SIDS actor system.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Error Handling](#error-handling)
3. [Response Handling](#response-handling)
4. [Actor Supervision](#actor-supervision)
5. [System Shutdown](#system-shutdown)
6. [Migration Guide](#migration-guide)

---

## Getting Started

SIDS is a Rust actor model framework for building concurrent applications. Install it:

```toml
[dependencies]
sids = "0.7"

# With optional features
sids = { version = "0.7", features = ["streaming", "visualize"] }
```

### Basic Example

```rust
use sids::actors::{
    actor::Actor,
    messages::{Message, ResponseMessage},
    start_actor_system, spawn_actor, send_message_by_id,
};

struct MyActor;

impl Actor<String, ResponseMessage> for MyActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        if let Some(payload) = message.payload {
            println!("Received: {}", payload);
        }
        if let Some(responder) = message.responder {
            responder.handle(ResponseMessage::Success).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut actor_system = start_actor_system::<String, ResponseMessage>();
    spawn_actor(&mut actor_system, MyActor, Some("MyActor".to_string())).await;
    
    let msg = Message {
        payload: Some("Hello!".to_string()),
        stop: false,
        responder: None,
        blocking: None,
    };
    
    send_message_by_id(&mut actor_system, 0, msg).await?;
    
    Ok(())
}
```

---

## Error Handling

**Since version 0.7.0**, SIDS uses proper error types instead of panics.

### Error Types

#### ActorError

```rust
pub enum ActorError {
    ActorNotFound { id: u32 },
    SendFailed { reason: String },
    ReceiveFailed { reason: String },
    ShutdownTimeout,
    GuardianNotResponding,
    InvalidState { reason: String },
    ChannelError { reason: String },
}
```

#### ConfigError

```rust
pub enum ConfigError {
    FileReadError { path: String, source: io::Error },
    ParseError { source: toml::de::Error },
    ValidationError { reason: String },
}
```

### Best Practices

#### 1. Use the `?` Operator

```rust
async fn process(actor_system: &mut ActorSystem<String, ResponseMessage>) -> Result<(), Box<dyn std::error::Error>> {
    send_message_by_id(actor_system, 1, message).await?;
    ping_actor_system(actor_system).await?;
    Ok(())
}
```

#### 2. Pattern Match for Specific Errors

```rust
match send_message_by_id(&mut actor_system, 1, message).await {
    Ok(()) => println!("Message sent"),
    Err(ActorError::ActorNotFound { id }) => {
        eprintln!("Actor {} not found, spawning new one", id);
        spawn_actor(&mut actor_system, MyActor, None).await;
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

#### 3. Graceful Degradation

```rust
let actor_ref = get_actor_sender(&actor_system, 1)
    .unwrap_or_else(|e| {
        eprintln!("Failed to get actor: {}", e);
        get_actor_sender(&actor_system, 0).expect("Guardian exists")
    });
```

#### 4. Top-Level Error Handling

```rust
#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    // Your application logic
    Ok(())
}
```

### Common Patterns

**Fire-and-forget messages:**

```rust
// In tests or examples where failure is not critical
send_message_by_id(&mut actor_system, 0, msg)
    .await
    .expect("Failed to send message");
```

**Production code:**

```rust
// Log and continue
if let Err(e) = send_message_by_id(&mut actor_system, 0, msg).await {
    error!("Failed to send message: {}", e);
}

// Or propagate
send_message_by_id(&mut actor_system, 0, msg).await?;
```

---

## Response Handling

SIDS uses the `ResponseHandler` trait to prevent memory leaks with async responses.

### Why ResponseHandler?

**Problem with raw oneshot channels:**

- Static lifetime requirements
- Memory leaks when receivers are dropped without awaiting
- No automatic cleanup

**Solution:** Trait-based abstraction with automatic cleanup.

### Basic Usage

```rust
use sids::actors::get_response_handler;

// Create a response handler
let (handler, rx) = get_response_handler::<ResponseMessage>();

let msg = Message {
    payload: Some("Hello".to_string()),
    stop: false,
    responder: Some(handler),  // ResponseHandler, not raw sender
    blocking: None,
};

send_message_by_id(&mut actor_system, 0, msg).await?;

// Wait for response
match rx.await {
    Ok(response) => println!("Received: {:?}", response),
    Err(e) => eprintln!("No response: {}", e),
}
// Handler automatically cleaned up when dropped
```

### Advanced Patterns

#### Shared Batch Handler

```rust
use sids::actors::response_handler::BatchResponseHandler;
use std::sync::Arc;

let (batch_handler, mut rx) = BatchResponseHandler::<ResponseMessage>::new(100);
let handler = Arc::new(batch_handler);

// Send multiple messages with shared handler
for i in 0..50 {
    let msg = Message {
        payload: Some(format!("Message {}", i)),
        stop: false,
        responder: Some(handler.clone()),
        blocking: None,
    };
    send_message_by_id(&mut actor_system, 0, msg).await?;
}

// Collect responses
while let Some(response) = rx.recv().await {
    println!("Got response: {:?}", response);
}
```

#### High-Throughput Pattern

```rust
// Send many messages without awaiting all
for i in 0..1000 {
    let (handler, rx) = get_response_handler::<ResponseMessage>();
    
    send_message_by_id(&mut actor_system, 0, msg).await?;
    
    // Only await some responses
    if i % 100 == 0 {
        let _ = rx.await;
    }
    // Other receivers dropped - no memory leak!
}
```

### In Actor Implementations

```rust
impl Actor<String, ResponseMessage> for MyActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        // Process message...
        
        // Send response if handler provided
        if let Some(responder) = message.responder {
            responder.handle(ResponseMessage::Success).await;
        }
    }
}
```

---

## Actor Supervision

Monitor and visualize your actor system with the `visualize` feature.

### Enable Supervision

```toml
[dependencies]
sids = { version = "0.7", features = ["visualize"] }
```

### Basic Usage

```rust
use sids::actors::start_actor_system_with_config;
use sids::config::SidsConfig;

let config = SidsConfig::default();
let mut actor_system = start_actor_system_with_config::<String, ResponseMessage>(config);

// Spawn actors (automatically tracked)
spawn_actor(&mut actor_system, MyActor, Some("MyActor".to_string())).await;

// Get supervision data
#[cfg(feature = "visualize")]
{
    let supervision = actor_system.get_supervision_data();
    let summary = actor_system.get_supervision_summary();
    
    println!("Active actors: {}", summary.active_actors);
    println!("Total messages: {}", summary.total_messages_processed);
}
```

### Export Formats

#### JSON

```rust
use sids::supervision_export::to_json;

let json = to_json(&supervision)?;
println!("{}", json);
```

#### Graphviz DOT

```rust
use sids::supervision_export::to_dot;

let dot = to_dot(&supervision);
// Save to file: graph.dot
// Visualize: dot -Tsvg graph.dot -o graph.svg
```

#### Mermaid Diagram

```rust
use sids::supervision_export::{to_mermaid, to_mermaid_sequence};

// State diagram
let mermaid = to_mermaid(&supervision);

// Sequence diagram
let sequence = to_mermaid_sequence(&supervision);

// Paste into https://mermaid.live
```

#### Text Summary

```rust
use sids::supervision_export::to_text_summary;

println!("{}", to_text_summary(&supervision));
```

### Metrics Tracked

**Per Actor:**

- Name and ID
- Status (Running, Stopped, Failed)
- Spawn time
- Messages received
- Last activity timestamp
- Recent event history

**System-wide:**

- Total actors spawned
- Active actors
- Total messages processed
- System uptime

### Recording Custom Events

```rust
#[cfg(feature = "visualize")]
actor_system.record_message_sent(from_actor_id, to_actor_id);
```

---

## System Shutdown

Gracefully shutdown your actor system with broadcast signals.

### Basic Shutdown

```rust
// Shutdown all actors
actor_system.shutdown().await?;
println!("System shut down gracefully");
```

### How It Works

1. Guardian receives shutdown signal
2. Broadcasts to all spawned actors
3. Each actor stops processing and exits
4. Waits for all actors to complete

### Subscribe to Shutdown Events

```rust
let mut shutdown_rx = actor_system.subscribe_shutdown();

tokio::spawn(async move {
    shutdown_rx.recv().await.ok();
    println!("System is shutting down");
    // Cleanup resources...
});
```

### Manual Stop Message

```rust
use sids::actors::response_handler::from_oneshot;

let (tx, rx) = tokio::sync::oneshot::channel();
let handler = from_oneshot(tx);

let stop_msg = Message {
    payload: None,
    stop: true,
    responder: Some(handler),
    blocking: None,
};

send_message_by_id(&mut actor_system, 0, stop_msg).await?;
rx.await?; // Wait for confirmation
```

### Timeout Configuration

```rust
use std::time::Duration;

let config = SidsConfig {
    shutdown_timeout: Duration::from_secs(10),
    ..Default::default()
};

let mut actor_system = start_actor_system_with_config(config);
```

### In Actor Implementations

Actors automatically stop when they receive the shutdown signal. No special handling needed:

```rust
impl Actor<String, ResponseMessage> for MyActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        // Normal message processing
        // Shutdown is handled automatically
    }
}
```

---

## Migration Guide

### From 0.6.x to 0.7.0

**Breaking Changes:**

- All public APIs now return `Result` types
- Response handlers use `ResponseHandler` trait instead of raw oneshot senders

#### API Changes

| Old (0.6.x) | New (0.7.0) |
| ----------- | ----------- |
| `send_message_by_id(&mut system, id, msg).await` | `send_message_by_id(&mut system, id, msg).await?` |
| `ping_actor_system(&system).await` | `ping_actor_system(&system).await?` |
| `get_actor_sender(&system, id)` | `get_actor_sender(&system, id)?` |
| `Config::from_toml_str(s)` | `Config::from_toml_str(s)?` |

#### Response Handlers

**Before (0.6.x):**

```rust
let (tx, rx) = tokio::sync::oneshot::channel();
let msg = Message {
    responder: Some(tx),  // Raw sender
    // ...
};
```

**After (0.7.0):**

```rust
use sids::actors::get_response_handler;

let (handler, rx) = get_response_handler::<ResponseMessage>();
let msg = Message {
    responder: Some(handler),  // ResponseHandler trait
    // ...
};
```

#### Error Handling

**Before - would panic:**

```rust
send_message_by_id(&mut system, 1, msg).await;
```

**After - proper error handling:**

```rust
// Option 1: Propagate
send_message_by_id(&mut system, 1, msg).await?;

// Option 2: Handle
match send_message_by_id(&mut system, 1, msg).await {
    Ok(()) => {},
    Err(e) => eprintln!("Send failed: {}", e),
}

// Option 3: Expect (for examples/tests)
send_message_by_id(&mut system, 1, msg)
    .await
    .expect("Send failed");
```

### Quick Migration Checklist

- [ ] Update `sids` to version 0.7+
- [ ] Replace raw oneshot senders with `get_response_handler()`
- [ ] Add `?` or error handling to all actor system calls
- [ ] Update function signatures to return `Result` where needed
- [ ] Test error paths
- [ ] Update examples/documentation

---

## Additional Resources

- [CHANGELOG.md](CHANGELOG.md) - Version history and changes
- [Examples](examples/) - Complete working examples
- [Architecture docs](architecture/) - System design documentation
- [API Documentation](https://docs.rs/sids) - Complete API reference

## Support

- Issues: [GitHub Issues](https://github.com/professor-greebie/sids/issues)
- Discussions: [GitHub Discussions](https://github.com/professor-greebie/sids/discussions)
- Security: See [SECURITY.md](SECURITY.md)
