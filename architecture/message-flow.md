# Message Flow Architecture

This document details how messages flow through the SIDS actor system, including patterns and best practices.

## Core Message Flow

### Basic Message Passing

```mermaid
sequenceDiagram
    participant A as Actor A
    participant S as ActorSystem
    participant R as ActorRef
    participant C as Channel
    participant B as Actor B

    A->>S: send_message_by_id(id, message)
    S->>S: Lookup actor by ID
    S->>R: Get ActorRef
    R->>C: sender.send(message)
    Note over C: Buffered channel<br/>(default: 100 messages)
    C->>B: Deliver message
    B->>B: receive(message)
    B->>B: Process message
```

### Message Structure

```rust
pub struct Message<MType, Response> {
    pub payload: Option<MType>,              // User data
    pub stop: bool,                          // Termination signal
    pub responder: Option<BoxedResponseHandler<Response>>,  // Response callback
    pub blocking: Option<()>,                // Sync hint
}
```

## Message Patterns

### 1. Fire-and-Forget

No response expected, fastest pattern.

```mermaid
sequenceDiagram
    participant Sender
    participant ActorSystem
    participant Receiver

    Sender->>ActorSystem: send_message_by_id(id, msg)
    Note over Sender: payload: Some(data)<br/>responder: None
    ActorSystem->>Receiver: Forward message
    Receiver->>Receiver: Process
    Note over Sender: Continue without waiting
```

**Code Example:**

```rust
let msg = Message {
    payload: Some(MyData::new()),
    stop: false,
    responder: None,
    blocking: None,
};

send_message_by_id(&mut actor_system, actor_id, msg).await?;
// Continue immediately
```

**Use Cases:**

- Logging
- Notifications
- Background tasks
- Non-critical operations

### 2. Request-Response

Sender waits for response.

```mermaid
sequenceDiagram
    participant Sender
    participant ActorSystem
    participant Receiver

    Sender->>Sender: Create response handler
    Sender->>ActorSystem: send_message(msg + handler)
    ActorSystem->>Receiver: Forward message
    Receiver->>Receiver: Process
    Receiver->>Sender: responder.handle(response)
    Sender->>Sender: await response
```

**Code Example:**

```rust
// Create response handler
let (handler, rx) = get_response_handler::<ResponseMessage>();

let msg = Message {
    payload: Some(Query::GetData),
    stop: false,
    responder: Some(handler),
    blocking: None,
};

send_message_by_id(&mut actor_system, actor_id, msg).await?;

// Wait for response
let response = rx.await?;
match response {
    ResponseMessage::Data(d) => println!("Got: {:?}", d),
    ResponseMessage::Error(e) => eprintln!("Error: {}", e),
    _ => {}
}
```

**Use Cases:**

- Query operations
- Validation requests
- Configuration retrieval
- State inquiries

### 3. Broadcast

One sender, multiple receivers.

```mermaid
graph TD
    Sender[Sender Actor]
    System[ActorSystem]
    R1[Receiver 1]
    R2[Receiver 2]
    R3[Receiver 3]

    Sender -->|send_message| System
    System -->|forward| R1
    System -->|forward| R2
    System -->|forward| R3

    style Sender fill:#e74c3c,color:#fff
    style R1 fill:#3498db,color:#fff
    style R2 fill:#3498db,color:#fff
    style R3 fill:#3498db,color:#fff
```

**Code Example:**

```rust
// Get all actor IDs
let actors = list_actors(&actor_system);

// Send to all
for (id, _name) in actors {
    let msg = Message {
        payload: Some(BroadcastData::Announcement("System update")),
        stop: false,
        responder: None,
        blocking: None,
    };
    send_message_by_id(&mut actor_system, id, msg).await?;
}
```

**Use Cases:**

- System announcements
- Configuration updates
- Event notifications
- Shutdown signals

### 4. Pipeline (Chain)

Sequential processing through multiple actors.

```mermaid
sequenceDiagram
    participant A as Actor A
    participant B as Actor B
    participant C as Actor C
    participant D as Actor D

    A->>B: Process step 1
    B->>B: Transform
    B->>C: Process step 2
    C->>C: Transform
    C->>D: Process step 3
    D->>D: Final result
    D->>A: Send result back
```

**Code Example:**

```rust
// Each actor forwards to next
impl Actor<PipelineMsg, ResponseMessage> for ProcessorA {
    async fn receive(&mut self, message: Message<PipelineMsg, ResponseMessage>) {
        if let Some(data) = message.payload {
            // Process
            let transformed = self.process(data);
            
            // Forward to next actor
            let next_msg = Message {
                payload: Some(transformed),
                stop: false,
                responder: message.responder,  // Pass along responder
                blocking: None,
            };
            send_message_by_id(&mut self.system, self.next_actor_id, next_msg).await.ok();
        }
    }
}
```

**Use Cases:**

- Data transformation pipelines
- Multi-stage validation
- ETL processes
- Request enrichment

### 5. Request-Response Pool

Multiple workers, one coordinator.

```mermaid
graph TB
    Coordinator[Coordinator Actor]
    W1[Worker 1]
    W2[Worker 2]
    W3[Worker 3]
    WN[Worker N]

    Coordinator -->|distribute| W1
    Coordinator -->|distribute| W2
    Coordinator -->|distribute| W3
    Coordinator -->|distribute| WN

    W1 -.response.-> Coordinator
    W2 -.response.-> Coordinator
    W3 -.response.-> Coordinator
    WN -.response.-> Coordinator

    style Coordinator fill:#9b59b6,color:#fff
    style W1 fill:#1abc9c,color:#fff
    style W2 fill:#1abc9c,color:#fff
    style W3 fill:#1abc9c,color:#fff
    style WN fill:#1abc9c,color:#fff
```

**Code Example:**

```rust
// Coordinator distributes work
let worker_ids = vec![worker1_id, worker2_id, worker3_id];
let mut next_worker = 0;

for task in tasks {
    let worker_id = worker_ids[next_worker];
    next_worker = (next_worker + 1) % worker_ids.len();  // Round-robin
    
    let (handler, rx) = get_response_handler();
    let msg = Message {
        payload: Some(task),
        stop: false,
        responder: Some(handler),
        blocking: None,
    };
    
    send_message_by_id(&mut actor_system, worker_id, msg).await?;
    responses.push(rx);  // Collect for later
}

// Wait for all responses
for rx in responses {
    let result = rx.await?;
    process_result(result);
}
```

**Use Cases:**

- Load balancing
- Parallel processing
- Task distribution
- Map-reduce patterns

## Channel Mechanics

### Buffered Channels

```mermaid
graph LR
    subgraph "Sender Side"
        S[Sender]
    end

    subgraph "Channel Buffer"
        B1[Slot 1]
        B2[Slot 2]
        B3[Slot 3]
        BN[Slot N]
    end

    subgraph "Receiver Side"
        R[Receiver]
    end

    S -->|send| B1
    B1 --> B2
    B2 --> B3
    B3 --> BN
    BN -->|receive| R

    style S fill:#e67e22,color:#fff
    style R fill:#27ae60,color:#fff
    style B1 fill:#95a5a6,color:#fff
    style B2 fill:#95a5a6,color:#fff
    style B3 fill:#95a5a6,color:#fff
    style BN fill:#95a5a6,color:#fff
```

**Behavior:**

- **Buffer Not Full**: `send()` returns immediately
- **Buffer Full**: `send()` blocks (awaits) until space available
- **Default Size**: 100 messages (configurable)

**Configuration:**

```toml
[actor_system]
actor_buffer_size = 200  # Increase for high-throughput
```

### Backpressure

```mermaid
sequenceDiagram
    participant Fast as Fast Producer
    participant Channel as Channel (Buffer: 3)
    participant Slow as Slow Consumer

    Fast->>Channel: send(msg1)
    Fast->>Channel: send(msg2)
    Fast->>Channel: send(msg3)
    Note over Channel: Buffer full
    Fast->>Channel: send(msg4)
    Note over Fast: Blocked, awaiting space
    Channel->>Slow: deliver(msg1)
    Note over Channel: Space available
    Note over Fast: Unblocked
    Channel->>Fast: send succeeds
```

**Best Practices:**

- Size buffers based on message rates
- Monitor channel capacity in production
- Use async sends (don't block critical paths)
- Consider priority queues for critical messages

## Response Patterns

### ResponseHandler Pattern (Recommended)

```mermaid
sequenceDiagram
    participant Sender
    participant Handler as ResponseHandler
    participant Receiver

    Sender->>Handler: Create handler
    Sender->>Receiver: Send msg + handler
    Receiver->>Handler: handler.handle(response)
    Handler->>Handler: Cleanup resources
    Handler->>Sender: Deliver response
    Note over Handler: Auto-cleanup on drop
```

**Benefits:**

- Automatic resource cleanup
- No memory leaks
- Type-safe
- Composable

**Code:**

```rust
let (handler, rx) = get_response_handler::<ResponseMessage>();

// Handler automatically cleans up when dropped
// Even if response never arrives
```

### Oneshot Channel Pattern

```mermaid
sequenceDiagram
    participant Sender
    participant Tx as oneshot::Sender
    participant Rx as oneshot::Receiver
    participant Receiver

    Sender->>Tx: Create oneshot channel
    Sender->>Receiver: Send message + tx
    Receiver->>Tx: tx.send(response)
    Tx->>Rx: Forward response
    Rx->>Sender: await response
```

**When to Use:**

- Simple request-response
- Known fast response
- Testing/examples

**Limitations:**

- Manual cleanup needed
- Can leak if not awaited
- Not composable

### Blocking Channel Pattern

```mermaid
sequenceDiagram
    participant Sync as Sync Code
    participant Tx as mpsc::SyncSender
    participant Rx as mpsc::Receiver
    participant Actor as Async Actor

    Sync->>Tx: Create blocking channel
    Sync->>Actor: Send message + tx
    Actor->>Tx: tx.send(response)
    Note over Sync: Blocks thread
    Tx->>Rx: Deliver
    Rx->>Sync: rx.recv()
    Note over Sync: Unblocked
```

**When to Use:**

- Bridging sync/async boundaries
- Main function without async
- FFI interfaces

**Warning:**

- Blocks OS thread
- Can cause deadlocks
- Use sparingly

## Message Routing

### By ID (Direct)

```mermaid
graph LR
    Sender[Sender]
    System[ActorSystem]
    Registry[Actor Registry]
    Actor[Target Actor]

    Sender -->|actor_id: 42| System
    System -->|lookup| Registry
    Registry -->|ActorRef| System
    System -->|forward| Actor
```

**Code:**

```rust
send_message_by_id(&mut actor_system, 42, message).await?;
```

**Characteristics:**

- O(1) lookup (HashMap)
- Fast and direct
- Requires knowing ID

### By Name (Indirect)

```mermaid
graph LR
    Sender[Sender]
    System[ActorSystem]
    NameMap[Name Map]
    Registry[Actor Registry]
    Actor[Target Actor]

    Sender -->|name: "worker"| System
    System -->|lookup name| NameMap
    NameMap -->|actor_id| System
    System -->|lookup id| Registry
    Registry -->|ActorRef| System
    System -->|forward| Actor
```

**Code:**

```rust
let actor_id = find_actor_by_name(&actor_system, "worker")?;
send_message_by_id(&mut actor_system, actor_id, message).await?;
```

**Characteristics:**

- Two-step lookup (name → ID → ActorRef)
- More flexible
- Useful for dynamic systems

### Using ActorRef (Cached)

```mermaid
graph LR
    Sender[Sender]
    CachedRef[Cached ActorRef]
    Channel[Direct Channel]
    Actor[Target Actor]

    Sender -->|use cached ref| CachedRef
    CachedRef -->|direct send| Channel
    Channel -->|deliver| Actor
```

**Code:**

```rust
// Cache the ActorRef
let actor_ref = get_actor_sender(&actor_system, actor_id)?;

// Use repeatedly without lookups
for msg in messages {
    actor_ref.send(msg).await?;
}
```

**Characteristics:**

- Zero lookup overhead
- Best performance
- Must handle actor lifecycle

## Error Handling in Messages

### Send Errors

```mermaid
graph TD
    Send[send_message_by_id]
    
    Send -->|Success| Ok[Ok result]
    Send -->|Actor not found| NotFound[ActorError::ActorNotFound]
    Send -->|Channel closed| SendFailed[ActorError::SendError]

    Ok --> Handle[User handles Ok]
    NotFound --> Handle
    SendFailed --> Handle

    style Ok fill:#2ecc71,color:#fff
    style NotFound fill:#e74c3c,color:#fff
    style SendFailed fill:#e74c3c,color:#fff
```

**Handling:**

```rust
match send_message_by_id(&mut actor_system, actor_id, msg).await {
    Ok(()) => println!("Message sent"),
    Err(ActorError::ActorNotFound(_)) => {
        eprintln!("Actor {} no longer exists", actor_id);
        // Handle gracefully
    }
    Err(ActorError::SendError(e)) => {
        eprintln!("Channel error: {}", e);
        // Retry or fail
    }
    Err(e) => eprintln!("Unexpected error: {:?}", e),
}
```

### Response Errors

```mermaid
graph TD
    Await[await response]
    
    Await -->|Success| Data[Response data]
    Await -->|Channel closed| RecvError[RecvError]
    Await -->|Timeout| Timeout[Timeout error]

    Data --> Process[Process response]
    RecvError --> Handle[Handle error]
    Timeout --> Handle

    style Data fill:#2ecc71,color:#fff
    style RecvError fill:#e74c3c,color:#fff
    style Timeout fill:#f39c12,color:#fff
```

**Handling:**

```rust
match tokio::time::timeout(Duration::from_secs(5), rx).await {
    Ok(Ok(response)) => {
        // Got response
        process(response);
    }
    Ok(Err(_)) => {
        // Channel closed (actor died)
        eprintln!("Actor terminated before responding");
    }
    Err(_) => {
        // Timeout
        eprintln!("Response timeout");
    }
}
```

## Performance Considerations

### Message Size

- **Small messages**: Fast, low overhead
- **Large messages**: Consider Arc for sharing
- **Clone cost**: Affects throughput

```rust
// Bad: Clone large data
struct HugeMessage(Vec<u8>);  // 10MB

// Better: Share with Arc
struct SharedMessage(Arc<Vec<u8>>);  // Clone is cheap
```

### Batching

```rust
// Instead of many small messages
for item in items {
    send_message(item).await?;  // N round-trips
}

// Batch them
send_message(BatchMessage(items)).await?;  // 1 round-trip
```

### Async vs Concurrent

```rust
// Sequential (slow)
for actor_id in actors {
    send_message(actor_id, msg.clone()).await?;
}

// Concurrent (fast)
let futures: Vec<_> = actors
    .iter()
    .map(|&actor_id| send_message(actor_id, msg.clone()))
    .collect();
tokio::join_all(futures).await;
```

## Best Practices

1. **Use Fire-and-Forget** when possible (lowest latency)
2. **Cache ActorRefs** for repeated sends
3. **Batch messages** to reduce overhead
4. **Use ResponseHandler** for request-response
5. **Handle errors explicitly** (don't unwrap)
6. **Avoid blocking** in receive methods
7. **Size buffers appropriately** for workload
8. **Monitor message counts** in production
9. **Use timeouts** for responses
10. **Profile before optimizing** (measure don't guess)

## Anti-Patterns

❌ **Blocking in receive:**

```rust
fn receive(&mut self, msg: Message) {
    std::thread::sleep(Duration::from_secs(10));  // Blocks Tokio!
}
```

✅ **Use async sleep:**

```rust
async fn receive(&mut self, msg: Message) {
    tokio::time::sleep(Duration::from_secs(10)).await;  // Yields
}
```

❌ **Ignoring send errors:**

```rust
send_message(id, msg).await.ok();  // Silent failure
```

✅ **Handle errors:**

```rust
send_message(id, msg).await
    .unwrap_or_else(|e| eprintln!("Send failed: {}", e));
```

❌ **Leaking response channels:**

```rust
let (tx, rx) = oneshot::channel();
send_message(msg, tx).await?;
// Never await rx - leaks memory!
```

✅ **Always await or drop explicitly:**

```rust
let (handler, rx) = get_response_handler();
send_message(msg, handler).await?;
let response = rx.await?;  // Or drop(rx) if not needed
```

## Next Steps

- See [actor-lifecycle.md](actor-lifecycle.md) for actor state management
- See [streaming-architecture.md](streaming-architecture.md) for reactive patterns
- See [actor-critic-pattern.md](actor-critic-pattern.md) for ML coordination example
