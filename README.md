# SIDS - An Actor Model Approach to Data Collection in RUST

This is an experimental actor-model system library built in rust. The repository has a few Mermaid diagrams 
and exxamples available for you to examine if you are interested in implementing the approach yourself.

## Getting Started

Run the example logging demonstration.

```
git clone https://github.com/professor-greebie/sids
cd sids
cargo run --example loggers
```

## What this does

This project demonstrates a simple approach to building actors in Rust, allowing for some abstraction between Tokio-type and blocking actors.

This is still a project in development, but it does illustrate how you might develop an actor system from scratch in Rust.

### Basic Concepts:

An actor - an actor implements an Actor<MType< Response>> Trait to include a `receive` function that accepts a message type of `Message<MType, Response>`.

The `Message` struct covers the most common Actor behaviors (stop, responses etc.), but you can add more as part of the payload, which is of type MType.

MType can be any base type (`String`, `u32` etc.) or an enum provided that it has Send features and can have static lifetime. Enums are powerful in Rust, so they are highly recommended. See the [Rust documentation on enum types for more information](https://doc.rust-lang.org/book/ch06-00-enums.html)

`Response` is any enum that the actors will use to send return messages back to the actor that sent them. A generic `ResponseMessage` can be used by default. 

Once you choose an MType, then the `ActorSystem` will use the same message type throughout the system.  Currently, only one MType is allowed, however, with Rust's enums, there is a lot of capacity for variance on the types of messages that can be sent.

```rust 
let mut actor_system = sids::actors::start_actor_system::<MType, Response>();
```

Starting an actor system initializes the system and runs a 'boss' actor called the `Guardian` with an id of 0. You can ping the boss using `sids::actors::ping_actor_system(&actor_system);`

You can add an actor to the system, by creating a structure that implements the Actor<MType> trait. All actors in the system must receive a Message<MType>.

```rust

use sids::actors::actor::Actor;
use sids::actors::messages::{Message, ResponseMessage};
use log::info;

#[derive(Debug, Clone)]
// Messages can be any valid enum that can derive Clone.
enum MyMessage {
    HELLO, GOODBYE, GHOST
}

// you can include some attributes like a name if you wish
struct MyActor;
// Actors must have static lifetime
impl Actor<MyMessage, ResponseMessage> for MyActor where MyActor: 'static  {
    async fn receive(&mut self, message: Message<MyMessage, ResponseMessage>) {
        if let Message { 
                // optional payload contains MyMessage
                payload, 
                // boolean that tells Actor whether it should stop after message.
                stop, 
                // optional tokio oneshot channel to send a response back to sender.
                // This response with a ResponseMessage that includes SUCCESS and FAILURE messages.
                responder, 
                // optional blocking channel to send a response back to sender if the Actor is intended to be blocking.
                blocking 
                } = message {
                    if let Some(msg) = payload {
                        info!("Message received {:?}", payload);
                    }
                    if let Some(respond) = responder {
                        respond.send(ResponseMessage::SUCCESS);
                    }
                }
    }
},

#[tokio::main]
async fn main() -> Result(()) {
    let my_actor = MyActor;


    let mut actor_system = sids::actors::start_actor_system::<MyMessage, ResponseMessage>().await;
    // gets a oneshot channel to receive a response from the system.
    let (tx, rx) = sids::actors::get_response_channel(&actor_system);
    let message = Message {
        payload: Some(MyMessage::HELLO),
        stop: false,
        responder: tx,
        blocking: None 
    }
    spawn_actor(&mut actor_system, my_actor, Some("My Actor".to_string())).await;
    // guardian is 0, so our actor id will be #1.
    send_message_by_id(&mut actor_system, 1, message).await;
    if let Ok(response) = rx.await {
        info!("Response received from actor {:?}", response);
    }

} 

## The Future

From a prototype perspective, this is final version of this project, except for performance and safety tweaks.

We will also include some more advanced examples, including using the Actor System to do Actor-Critic Machine Learning work.

## Citations

The following resources helped me a lot during the building of this demonstration.


- Mara Bos (2023) *Rust Atomics and Locks: Low-level concurrency in Practice.* O'Reilly Press.
- Alice Ryhl (2021) [*Actors with Tokio*](https://ryhl.io/blog/actors-with-tokio/) [Blog Post].