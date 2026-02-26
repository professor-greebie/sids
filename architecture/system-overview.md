# System Architecture Overview

This document provides a high-level overview of SIDS architecture and core components.

## High-Level Architecture

```mermaid
graph TB
    subgraph "User Application"
        UserCode[User Code]
    end

    subgraph "SIDS Public API"
        API[Public Functions<br/>start_actor_system<br/>spawn_actor<br/>send_message<br/>etc.]
    end

    subgraph "Actor System Core"
        System[ActorSystem&lt;MType, Response&gt;]
        Registry[Actor Registry<br/>HashMap&lt;u32, ActorRef&gt;]
        NameMap[Name Mapping<br/>HashMap&lt;String, u32&gt;]
        Counter[Actor ID Counter<br/>AtomicU32]
    end

    subgraph "Actors"
        Guardian[Guardian Actor<br/>ID: 0]
        Actor1[Custom Actor<br/>ID: 1]
        Actor2[Custom Actor<br/>ID: 2]
        ActorN[Custom Actor<br/>ID: N]
    end

    subgraph "Channel Layer"
        Chan1[tokio::mpsc::channel]
        Chan2[tokio::mpsc::channel]
        ChanN[tokio::mpsc::channel]
    end

    subgraph "Async Runtime"
        Tokio[Tokio Runtime<br/>Thread Pool]
    end

    UserCode --> API
    API --> System
    System --> Registry
    System --> NameMap
    System --> Counter
    
    Registry -.ActorRef.-> Guardian
    Registry -.ActorRef.-> Actor1
    Registry -.ActorRef.-> Actor2
    Registry -.ActorRef.-> ActorN

    Guardian --> Chan1
    Actor1 --> Chan2
    ActorN --> ChanN

    Chan1 --> Tokio
    Chan2 --> Tokio
    ChanN --> Tokio

    style System fill:#4a90e2,color:#fff
    style Guardian fill:#e27d60,color:#fff
    style Actor1 fill:#85dcb0,color:#000
    style Actor2 fill:#85dcb0,color:#000
    style ActorN fill:#85dcb0,color:#000
```

## Core Components

### 1. ActorSystem

The central coordinator managing all actors and system state.

**Responsibilities:**

- Actor registration and ID assignment
- Name-to-ID mapping
- Message routing
- Channel creation
- System monitoring (message/thread counts)

**Key Fields:**

```rust
pub struct ActorSystem<MType, Response> {
    actor_refs: Arc<Mutex<HashMap<u32, ActorRef<MType, Response>>>>,
    actor_names: Arc<Mutex<HashMap<String, u32>>>,
    next_actor_id: Arc<AtomicU32>,
    channel_factory: Arc<ChannelFactory<MType, Response>>,
    // ... monitoring fields
}
```

**Thread Safety:**

- Uses `Arc` for shared ownership
- `Mutex` for synchronized access to collections
- `AtomicU32` for lock-free ID generation

### 2. Actor Trait

Defines the behavior interface all actors must implement.

```rust
#[async_trait]
pub trait Actor<MType, Response>: Send + 'static {
    async fn receive(&mut self, message: Message<MType, Response>);
}
```

**Key Properties:**

- `Send`: Can be transferred between threads
- `'static`: No borrowed references (owned data only)
- `async`: Non-blocking message processing

### 3. ActorRef

A reference (handle) to an actor for sending messages.

```rust
pub struct ActorRef<MType, Response> {
    sender: Sender<Message<MType, Response>>,
    actor_id: u32,
}
```

**Capabilities:**

- Send messages to actor
- Non-blocking (buffered channel)
- Clone-able (multiple references to same actor)

### 4. Message

The message envelope carrying data and metadata.

```rust
pub struct Message<MType, Response> {
    pub payload: Option<MType>,
    pub stop: bool,
    pub responder: Option<BoxedResponseHandler<Response>>,
    pub blocking: Option<()>,
}
```

**Fields:**

- **payload**: User-defined message data
- **stop**: Signal to terminate actor
- **responder**: Optional response handler
- **blocking**: Hint for synchronous processing

### 5. Guardian Actor

Special actor (ID 0) automatically spawned with every system.

**Purpose:**

- System heartbeat (responds to ping)
- Default message receiver
- System initialization

## Component Interactions

### Actor Creation Flow

```mermaid
sequenceDiagram
    participant User
    participant API as Public API
    participant System as ActorSystem
    participant Runtime as Tokio
    participant Actor

    User->>API: spawn_actor(system, actor, name)
    API->>System: spawn_actor(actor, name)
    System->>System: Generate actor ID
    System->>Runtime: tokio::spawn(async task)
    Runtime->>Actor: Start receive loop
    System->>System: Store ActorRef in registry
    System->>System: Map name -> ID
    System-->>User: Actor spawned (ID assigned)
```

### Message Sending Flow

```mermaid
sequenceDiagram
    participant Sender as Sender Actor
    participant System as ActorSystem
    participant Channel as mpsc::channel
    participant Receiver as Receiver Actor

    Sender->>System: send_message_by_id(id, message)
    System->>System: Lookup ActorRef by ID
    System->>Channel: sender.send(message)
    Note over Channel: Buffered queue
    Channel->>Receiver: Message delivered
    Receiver->>Receiver: actor.receive(message)
    
    alt With Response
        Receiver->>Sender: responder.handle(response)
    end
```

## System Startup Sequence

```mermaid
sequenceDiagram
    participant User
    participant API as start_actor_system()
    participant System as ActorSystem
    participant Guardian as Guardian Actor

    User->>API: start_actor_system()
    API->>System: new()
    System->>System: Initialize registries
    System->>System: Create channel factory
    System->>System: Set actor_id = 0
    System->>Guardian: Spawn guardian
    Guardian->>Guardian: Start receive loop
    System-->>User: Return ActorSystem
    Note over User,System: System ready for actors
```

## Memory Model

### Ownership

```mermaid
graph LR
    subgraph "Actor System"
        System[ActorSystem<br/>Arc&lt;Mutex&lt;HashMap&gt;&gt;]
    end

    subgraph "Actor Refs"
        Ref1[ActorRef<br/>Clone 1]
        Ref2[ActorRef<br/>Clone 2]
        RefN[ActorRef<br/>Clone N]
    end

    subgraph "Actors"
        A1[Actor 1<br/>Owned State]
        A2[Actor 2<br/>Owned State]
    end

    System -.arc.-> Ref1
    System -.arc.-> Ref2
    System -.arc.-> RefN

    Ref1 --> A1
    Ref2 --> A2

    style System fill:#f9ca24
    style A1 fill:#6ab04c
    style A2 fill:#6ab04c
```

**Key Points:**

- Each actor owns its state (no shared memory)
- `ActorRef` clones allow multiple senders
- System holds authoritative registry
- Channels provide message isolation

## Concurrency Model

### Threading

```mermaid
graph TB
    subgraph "Tokio Runtime"
        T1[Worker Thread 1]
        T2[Worker Thread 2]
        TN[Worker Thread N]
    end

    subgraph "Actor Tasks"
        A1[Actor 1 Task]
        A2[Actor 2 Task]
        A3[Actor 3 Task]
        AN[Actor N Task]
    end

    T1 -.schedules.-> A1
    T1 -.schedules.-> A2
    T2 -.schedules.-> A3
    TN -.schedules.-> AN

    A1 -.message.-> A2
    A2 -.message.-> A3
    A3 -.message.-> AN
```

**Characteristics:**

- Actors run as independent async tasks
- Tokio schedules tasks across thread pool
- No locks needed for actor state (isolated)
- Message channels handle synchronization

## Error Handling

```mermaid
graph TD
    Operation[Actor Operation]
    
    Operation -->|Success| Ok[Return Ok&lt;T&gt;]
    Operation -->|Actor Not Found| NotFound[ActorError::ActorNotFound]
    Operation -->|Send Failed| SendError[ActorError::SendError]
    Operation -->|Config Error| ConfigError[ActorError::ConfigError]
    
    Ok --> User[User Handles Result]
    NotFound --> User
    SendError --> User
    ConfigError --> User

    style Ok fill:#2ecc71,color:#fff
    style NotFound fill:#e74c3c,color:#fff
    style SendError fill:#e74c3c,color:#fff
    style ConfigError fill:#e74c3c,color:#fff
```

All public APIs return `ActorResult<T>` allowing proper error handling with `?` operator.

## Configuration

```mermaid
graph LR
    Config[SidsConfig]
    File[sids.config.toml]
    Default[Default Values]

    File -->|load| Config
    Default -->|fallback| Config
    Config -->|initialize| System[ActorSystem]

    style Config fill:#3498db,color:#fff
```

**Configurable Parameters:**

- Actor buffer size (channel capacity)
- Shutdown timeout
- Custom system settings

## Monitoring

```mermaid
graph TB
    System[ActorSystem]
    
    System --> MsgCount[Message Counter<br/>AtomicUsize]
    System --> ThreadCount[Thread Counter<br/>AtomicUsize]
    
    MsgCount --> API1[get_message_count_reference]
    MsgCount --> API2[get_total_messages]
    ThreadCount --> API3[get_thread_count_reference]
    ThreadCount --> API4[get_total_threads]

    style System fill:#9b59b6,color:#fff
    style MsgCount fill:#1abc9c,color:#fff
    style ThreadCount fill:#1abc9c,color:#fff
```

Real-time monitoring via atomic counters (lock-free reads).

## Design Trade-offs

### Strengths

- **Type Safety**: Compile-time guarantees via Rust type system
- **Memory Safety**: No data races, no memory leaks (RAII)
- **Performance**: Lock-free where possible, efficient async
- **Scalability**: Linear scaling with cores

### Trade-offs

- **Single Message Type**: All actors use same `MType` (use enums for variety)
- **No Actor Hierarchy**: Flat structure (supervision optional)
- **Local Only**: No built-in distributed actors (v1.0)
- **Tokio Dependent**: Tied to Tokio async runtime

## Comparison with Other Frameworks

| Feature | SIDS | Actix | Tokio Actors |
| ------- | ---- | ----- | ------------ |
| Type Safety | Strong | Strong | Strong |
| Message Types | Single (enum-based) | Multiple | Flexible |
| Supervision | Optional | Built-in | Manual |
| Streaming | First-class | Via Actix-web | Manual |
| Learning Curve | Low | Medium | Low |
| Production Ready | v1.0+ | Yes | DIY |

## Next Steps

- See [message-flow.md](message-flow.md) for detailed message passing patterns
- See [actor-lifecycle.md](actor-lifecycle.md) for actor state transitions
- See [streaming-architecture.md](streaming-architecture.md) for reactive streams
