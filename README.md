# SIDS - An Actor Model Approach to Data Collection in RUST

This is an experimental actor-model system library built in rust. The repository has a few Mermaid diagrams 
and exxamples available for you to examine if you are interested in implementing the approach yourself.

## Getting Started

Run the example logging demonstration.

```
git clone https://github.com/professor-greebie/sids
cd sids
cargo run --example logger
```

## What this does

This project demonstrates a simple approach to building actors in Rust, allowing for some abstraction between Tokio-type and blocking actors.

This is still a project in development, but it does illustrate how you might develop an actor system from scratch in Rust.

Basic Concepts:

An actor - an actor implements an ActorTrait to include an async `receive` function.

To use the actor, you need to start an actor system. That's pretty straight-forward
using the sids::actors api.

```rust 
let mut actor_system = sids::actors::api::start_actor_system();
```

Starting an actor system initializes the system and runs a 'boss' actor called the `Guardian`.

You can add an actor to the system, by creating an officer. In order to do this, you need to create a structure that implements the ActorTrait trait. For now, the receive function only accepts InternalMessage, which is an enum with a few preset messages. Some messages in InternalMessage include a responder that allows the actor
to notify a higher level actor (usually the guardian) about the result, often a success or failure.

```rust

use sids::actors::actor::Actor;


// you can include some attributes like a name if you wish
struct MyActor;
impl Actor for MyActor where MyActor: 'static {
    // in future this may not need to be async
    async fn receive(&mut self, message: Message) {
        // The `Message` enum carries all possible messages in the system.
        // Customization may be available in the future, but for now, it is also possible
        // to use the StringMessage variant to provide custom messages.
        match message {
            Message::StringMessage { string } => 
                // custom options here
                // as in "if string == 'hello' => println("hello there friend");"
                info!("Received message {}", string),
            Message::RequestStatus { responder } {
                responder.send(ResponseMessage::Ok)
            }
        },
    }
}

let my_actor = MyActor;

sids::actors::api::spawn_officer(
    &mut actor_system, // the system you are adding your actor to
    Some("My actor name"), // an optional name for easy reference
    my_actor // your implementated actor
).await
```

Officers will be kept in a vector in the GuardianActor, so its id will be as per the index.
In future there will be a better approach to capturing actors by name or type. To send a message to this 
actor, you simple do:

```rust

send_message_to_officer_enum(
    &actor_system, // the actor system
    0, // the officer id, currently a simple Vec index for now
    Message::RequestStatus, // the message you wish to send
    false // whether the actor is a thread-blocking type (see below)
    );

```

Actors may also be blocking in case you need that for collecting data from an http response or something else 
that requires thread blocking in order to operate. Blocking actors are kept in their own Vector, so 
indices will need to account for that (for now).

## The Future

This project is still in development, so any of these ideas could be added or dropped at any time.

- Courriers - these are actors that hold state that can be updated by Officer actors (that will also act as an Observer).
- Custom messaging - currently, actors only accept "InternalMessage" which is limiting if one wants to create their own messaging system. Among the challenges producing custom message is that responders cannot be 
cloned.
- A collection of Actors for doing typical ETL things like Kafka consumer and producers, database, conversion, 
serde etc.

## Citations

The following resources helped me a lot during the building of this demonstration.


- Mara Bos (2023) *Rust Atomics and Locks: Low-level concurrency in Practice.* O'Reilly Press.
- Alice Ryhl (2021) [*Actors with Tokio*](https://ryhl.io/blog/actors-with-tokio/) [Blog Post].