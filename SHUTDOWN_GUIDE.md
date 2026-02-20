# Actor System Shutdown Guide

## Overview

The Actor System now supports graceful system-wide shutdown through a broadcast signal mechanism. When the Guardian receives a stop message, it broadcasts a shutdown signal to all spawned actors, allowing them to terminate cleanly.

## Architecture

The shutdown mechanism uses Tokio's broadcast channel:

1. **Guardian**: When it receives a message with `stop: true`, it broadcasts a shutdown signal to all actors
2. **Broadcast Channel**: All spawned actors subscribe to receive the shutdown signal
3. **Graceful Shutdown**: Each actor stops processing messages and exits its main loop upon receiving the signal

## Usage

### Basic Shutdown

```rust
use sids::actors::{start_actor_system, spawn_actor, Actor};
use sids::actors::messages::{Message, ResponseMessage};

#[derive(Clone)]
struct MyMessage(String);

struct MyActor;

impl Actor<MyMessage, ResponseMessage> for MyActor {
    async fn receive(&mut self, message: Message<MyMessage, ResponseMessage>) {
        if let Some(payload) = message.payload {
            println!("Received: {}", payload.0);
        }
    }
}

#[tokio::main]
async fn main() {
    let mut actor_system = start_actor_system::<MyMessage, ResponseMessage>();
    
    // Spawn actors
    let actor = MyActor;
    spawn_actor(&mut actor_system, actor, Some("MyActor".to_string())).await;
    
    // Do work...
    
    // Shutdown the entire system
    actor_system.shutdown().await.expect("Shutdown failed");
    println!("System shutting down");
}
```

### Subscribing to Shutdown Events

```rust
// Subscribe to system shutdown events
let mut shutdown_rx = actor_system.subscribe_shutdown();

// In another task, wait for shutdown
tokio::spawn(async move {
    shutdown_rx.recv().await.ok();
    println!("System has shut down");
});
```

### Manual Shutdown Control

You can also send a stop message directly to the guardian:

```rust
let (tx, rx) = tokio::sync::oneshot::channel();
let stop_msg = Message {
    payload: None,
    stop: true,
    responder: Some(tx),
    blocking: None,
};

actor_system.snd.send(stop_msg).await?;
let _response = rx.await?; // Wait for guardian to confirm
```

## How It Works

### When ActorSystem Shuts Down

1. **Send Stop Message**: A stop message is sent to the Guardian
2. **Guardian Broadcasts**: The Guardian broadcasts the shutdown signal via the broadcast channel
3. **Actors Close**: All spawned actors receive the shutdown signal and exit their message processing loops
4. **System Exits**: The system terminates once all actors have stopped

### Actor Message Processing Loop

```rust
// Simplified pseudo-code of the actor loop
loop {
    tokio::select! {
        Some(message) = actor.receiver.recv() => {
            if message.stop {
                break;  // Stop on explicit stop message
            }
            actor.receive(message).await;
        }
        _ = shutdown_rx.recv() => {
            // Shutdown signal received
            break;
        }
    }
}
```

## Key Features

- **Broadcast-based**: All actors receive the shutdown signal simultaneously
- **Non-blocking**: Shutdown is triggered via message passing, not forceful termination
- **Graceful**: Actors can clean up resources before exiting
- **Observable**: Clients can subscribe to shutdown events using `subscribe_shutdown()`
- **Compatible**: Existing code continues to work; shutdown is optional

## Best Practices

1. **Always await shutdown**: Use `actor_system.shutdown().await` to cleanly stop the system
2. **Subscribe early**: Subscribe to shutdown signals before starting critical work
3. **Handle timeouts**: Add timeouts when waiting for shutdown confirmation
4. **Resource cleanup**: Implement cleanup logic in actor drop or before the loop exits

## Example: Graceful Server Shutdown

```rust
#[tokio::main]
async fn main() {
    let mut actor_system = start_actor_system::<Request, Response>();
    
    // Spawn worker actors
    for i in 0..10 {
        spawn_actor(&mut actor_system, WorkerActor, Some(format!("worker-{}", i))).await;
    }
    
    // Subscribe to shutdown
    let mut shutdown_rx = actor_system.subscribe_shutdown();
    
    // Start server (simplified)
    let server_task = tokio::spawn(async {
        // Server loop
    });
    
    // Wait for shutdown signal or server completion
    tokio::select! {
        _ = server_task => {
            println!("Server stopped");
        }
        _ = shutdown_rx.recv() => {
            println!("Shutdown requested, stopping server...");
        }
    }
    
    // Cleanly shutdown remaining actors
    actor_system.shutdown().await.ok();
}
```

## Implementation Details

### Files Modified

- [src/actors/actor_system.rs](src/actors/actor_system.rs): Added broadcast channel and shutdown methods
- [src/actors/actor.rs](src/actors/actor.rs): Updated actor loops to listen for shutdown signals
- [src/actors/messages.rs](src/actors/messages.rs): No changes (existing `stop` flag used)

### API Changes

**New Methods**:

- `ActorSystem::shutdown() -> Result<(), String>`
- `ActorSystem::subscribe_shutdown() -> broadcast::Receiver<()>`

**Modified Structures**:

- `Guardian`: Now holds `shutdown_tx: Option<broadcast::Sender<()>>`
- `ActorImpl` and `BlockingActorImpl`: Now hold `shutdown_rx: Option<broadcast::Receiver<()>>`

All changes are backward compatible with existing code.
