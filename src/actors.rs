//! Core actor system APIs.
//!
//! This module provides the stable public API for SIDS actor systems.
//! All APIs in this module are stable as of v1.0.0 and follow semantic versioning.
//!
//! For stability guarantees, see [docs/STABILITY.md](../../docs/STABILITY.md).

pub mod actor;
pub mod actor_ref;
pub mod actor_system;
pub mod channel_factory;
pub mod error;
pub mod messages;
pub mod response_handler;

use crate::config::{SidsConfig, DEFAULT_ACTOR_BUFFER_SIZE};
use actor::Actor;
use actor_ref::ActorRef;
use actor_system::ActorSystem;
use channel_factory::ChannelFactory;
pub use error::{ActorError, ActorResult};
use messages::Message;
use response_handler::{from_oneshot, BoxedResponseHandler};

pub const SIDS_DEFAULT_BUFFER_SIZE: usize = DEFAULT_ACTOR_BUFFER_SIZE;

/// Creates an actor system with the default configuration. The guardian actor is automatically spawned with ID 0.
pub fn start_actor_system<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
) -> ActorSystem<MType, Response> {
    ActorSystem::<MType, Response>::new()
}

/// Creates an actor system with a custom configuration. The guardian actor is automatically spawned with ID 0.
pub fn start_actor_system_with_config<
    MType: Send + Clone + 'static,
    Response: Send + Clone + 'static,
>(
    config: SidsConfig,
) -> ActorSystem<MType, Response> {
    ActorSystem::<MType, Response>::new_with_config(config)
}

/// Spawns an actor in the `actor_system`. If a name is provided, it will be used for logging and debugging purposes. The actor will be assigned a unique ID by the system.
pub async fn spawn_actor<MType: Send + Clone + 'static, Response: Send + Clone + 'static, T>(
    actor_system: &mut ActorSystem<MType, Response>,
    actor: T,
    name: Option<String>,
) where
    T: Actor<MType, Response> + 'static,
{
    actor_system.spawn_actor(actor, name).await;
}

/// Spawns an actor that uses blocking.
pub async fn spawn_blocking_actor<
    MType: Send + Clone + 'static,
    Response: Send + Clone + 'static,
    T,
>(
    actor_system: &mut ActorSystem<MType, Response>,
    actor: T,
    name: Option<String>,
) where
    T: Actor<MType, Response> + 'static,
{
    actor_system.spawn_blocking_actor(actor, name);
}

/// Sends a message to the actor with a specified ID.
/// Returns an error if the actor does not exist. The guardian actor (ID 0) is always available.
pub async fn send_message_by_id<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &mut ActorSystem<MType, Response>,
    actor_id: u32,
    message: Message<MType, Response>,
) -> ActorResult<()> {
    actor_system.send_message_to_actor(actor_id, message).await
}

/// Pings the actor system to verify it is responsive. Returns an error if the guardian is unreachable.
pub async fn ping_actor_system<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
) -> ActorResult<()> {
    actor_system.ping_system().await
}

/// Creates a new oneshot channel for receiving a response.
/// The sender can be used to send a response back to the caller, and the receiver can be awaited to get the response.
/// This is useful for request-response patterns where an actor needs to send a message and wait for a reply.
pub fn get_response_channel<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
) -> (
    tokio::sync::oneshot::Sender<Response>,
    tokio::sync::oneshot::Receiver<Response>,
) {
    actor_system.create_response_channel()
}

/// Creates a new response handler from a oneshot channel.
/// This is the preferred approach as it prevents memory leaks when many responses are sent.
/// The handler automatically manages cleanup when dropped.
///
/// # Example
/// ```rust
/// use sids::actors::get_response_handler;
/// use sids::actors::messages::ResponseMessage;
///
/// let (handler, rx) = get_response_handler::<ResponseMessage>();
/// // Use handler in Message::responder field
/// // When done, the handler will clean up automatically when the Arc is dropped
/// ```
pub fn get_response_handler<Response: Send + Clone + std::fmt::Debug + 'static>() -> (
    BoxedResponseHandler<Response>,
    tokio::sync::oneshot::Receiver<Response>,
) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let handler = from_oneshot(tx);
    (handler, rx)
}

/// Creates a new blocking channel for receiving a response.
/// The sender can be used to send a response back to the caller, and the receiver can be used to blockingly wait for the response.
/// This is useful for scenarios where the caller cannot use async/await and needs to block until a response is received.
pub fn get_blocking_response_channel<
    MType: Send + Clone + 'static,
    Response: Send + Clone + 'static,
>(
    actor_system: &ActorSystem<MType, Response>,
) -> (
    std::sync::mpsc::SyncSender<Response>,
    std::sync::mpsc::Receiver<Response>,
) {
    actor_system.create_blocking_response_channel()
}

/// Retrieves an `ActorRef` for the actor with the specified ID.
/// Returns an error if the actor does not exist.
pub fn get_actor_sender<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
    id: u32,
) -> ActorResult<ActorRef<MType, Response>> {
    actor_system.get_actor_ref(id)
}

/// Retrieves a reference to the atomic message count for the actor system. This allows you to monitor the number of messages processed by the system in real-time.
pub fn get_message_count_reference<
    MType: Send + Clone + 'static,
    Response: Send + Clone + 'static,
>(
    actor_system: &ActorSystem<MType, Response>,
) -> &'static std::sync::atomic::AtomicUsize {
    actor_system.get_message_count_reference()
}

/// Retrieves a reference to the atomic thread count for the actor system. This allows you to monitor the number of active threads in the system in real-time.
pub fn get_thread_count_reference<
    MType: Send + Clone + 'static,
    Response: Send + Clone + 'static,
>(
    actor_system: &ActorSystem<MType, Response>,
) -> &'static std::sync::atomic::AtomicUsize {
    actor_system.get_thread_count_reference()
}

/// Retrieves the total number of messages processed by the actor system. This is a snapshot of the message count at the time of the call and may not reflect real-time changes.
///
pub fn get_total_messages<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
) -> usize {
    actor_system.get_message_count()
}
/// Retrieves the total number of active threads in the actor system. This is a snapshot of the thread count at the time of the call and may not reflect real-time changes.
pub fn get_total_threads<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
) -> usize {
    actor_system.get_thread_count()
}

/// Stops a specific actor by sending it a stop message.
/// Returns an error if the actor does not exist.
///
/// # Example
/// ```rust
/// use sids::actors;
///
/// # async fn example() {
/// # let mut actor_system = actors::start_actor_system::<String, actors::messages::ResponseMessage>();
/// // Stop actor with ID 1
/// actors::stop_actor(&mut actor_system, 1).await.ok();
/// # }
/// ```
pub async fn stop_actor<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &mut ActorSystem<MType, Response>,
    actor_id: u32,
) -> ActorResult<()> {
    actor_system.stop_actor(actor_id).await
}

/// Lists all actors in the system with their IDs and names.
/// Returns a vector of (id, name) tuples sorted by ID.
///
/// # Example
/// ```rust
/// use sids::actors;
///
/// # async fn example() {
/// # let actor_system = actors::start_actor_system::<String, actors::messages::ResponseMessage>();
/// let actors = actors::list_actors(&actor_system);
/// for (id, name) in actors {
///     println!("Actor {}: {}", id, name);
/// }
/// # }
/// ```
pub fn list_actors<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
) -> Vec<(u32, String)> {
    actor_system.list_actors()
}

/// Finds an actor ID by its name.
/// Returns an error if no actor with the given name exists.
///
/// # Example
/// ```rust
/// use sids::actors;
///
/// # async fn example() {
/// # let mut actor_system = actors::start_actor_system::<String, actors::messages::ResponseMessage>();
/// match actors::find_actor_by_name(&actor_system, "MyActor") {
///     Ok(id) => println!("Found actor at ID {}", id),
///     Err(_) => println!("Actor not found"),
/// }
/// # }
/// ```
pub fn find_actor_by_name<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
    name: &str,
) -> ActorResult<u32> {
    actor_system.find_actor_by_name(name)
}

/// Checks if an actor with the given ID exists in the system.
///
/// # Example
/// ```rust
/// use sids::actors;
///
/// # async fn example() {
/// # let actor_system = actors::start_actor_system::<String, actors::messages::ResponseMessage>();
/// if actors::actor_exists(&actor_system, 1) {
///     println!("Actor 1 exists");
/// }
/// # }
/// ```
pub fn actor_exists<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(
    actor_system: &ActorSystem<MType, Response>,
    actor_id: u32,
) -> bool {
    actor_system.actor_exists(actor_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;
    use messages::ResponseMessage;

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum TestMessage {
        Ping,
        Echo(String),
        Increment,
    }

    struct TestActor {
        name: String,
        counter: u32,
    }

    impl Actor<TestMessage, ResponseMessage> for TestActor {
        async fn receive(&mut self, message: Message<TestMessage, ResponseMessage>) {
            info!("TestActor '{}' received message", self.name);
            if message.stop {
                return;
            }

            if let Some(payload) = message.payload {
                match payload {
                    TestMessage::Ping => {
                        info!("TestActor received Ping");
                        if let Some(responder) = message.responder {
                            responder.handle(ResponseMessage::Success).await;
                        }
                    }
                    TestMessage::Echo(text) => {
                        info!("TestActor received Echo: {}", text);
                        self.counter += 1;
                    }
                    TestMessage::Increment => {
                        self.counter += 1;
                        info!("TestActor counter: {}", self.counter);
                    }
                }
            }
        }
    }

    struct BlockingTestActor {
        name: String,
    }

    impl Actor<TestMessage, ResponseMessage> for BlockingTestActor {
        async fn receive(&mut self, message: Message<TestMessage, ResponseMessage>) {
            info!("BlockingTestActor '{}' received message", self.name);
            if let Some(blocking) = message.blocking {
                let _ = blocking.send(ResponseMessage::Success);
            }
        }
    }

    #[tokio::test]
    async fn test_start_actor_system() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Ping the guardian to verify system is working
        ping_actor_system(&actor_system)
            .await
            .expect("Ping should succeed");
    }

    #[tokio::test]
    async fn test_spawn_actor() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let test_actor = TestActor {
            name: "TestActor1".to_string(),
            counter: 0,
        };

        spawn_actor(
            &mut actor_system,
            test_actor,
            Some("test_actor".to_string()),
        )
        .await;

        // Test passes if no panic occurs during spawn
    }

    #[tokio::test]
    async fn test_spawn_actor_without_name() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let test_actor = TestActor {
            name: "UnnamedActor".to_string(),
            counter: 0,
        };

        spawn_actor(&mut actor_system, test_actor, None).await;

        // Test passes if no panic occurs during spawn
    }

    #[tokio::test]
    async fn test_spawn_blocking_actor() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let blocking_actor = BlockingTestActor {
            name: "BlockingActor1".to_string(),
        };

        spawn_blocking_actor(
            &mut actor_system,
            blocking_actor,
            Some("blocking_test".to_string()),
        )
        .await;

        // Test passes if no panic occurs during spawn
    }

    #[tokio::test]
    async fn test_send_message_by_id() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let test_actor = TestActor {
            name: "MessageTarget".to_string(),
            counter: 0,
        };

        spawn_actor(&mut actor_system, test_actor, Some("target".to_string())).await;

        let message = Message {
            payload: Some(TestMessage::Increment),
            stop: false,
            responder: None,
            blocking: None,
        };

        // Send to guardian (ID 0) - should work
        send_message_by_id(&mut actor_system, 0, message)
            .await
            .expect("Send should succeed");

        // Give the system time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_ping_actor_system() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Ping should succeed without panic
        ping_actor_system(&actor_system)
            .await
            .expect("Ping should succeed");
    }

    #[tokio::test]
    async fn test_get_response_channel() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let (tx, rx) = get_response_channel(&actor_system);

        // Send a response through the channel
        tx.send(ResponseMessage::Success).unwrap();

        // Receive the response
        let response = rx.await.unwrap();
        assert!(matches!(response, ResponseMessage::Success));
    }

    #[tokio::test]
    async fn test_get_response_handler() {
        let (handler, rx) = get_response_handler::<ResponseMessage>();

        // Send a response through the handler
        handler.handle(ResponseMessage::Success).await;

        // Receive the response
        let response = rx.await.unwrap();
        assert!(matches!(response, ResponseMessage::Success));
    }

    #[tokio::test]
    async fn test_get_blocking_response_channel() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let (tx, rx) = get_blocking_response_channel(&actor_system);

        // Send a response through the blocking channel
        tx.send(ResponseMessage::Success).unwrap();

        // Receive the response
        let response = rx.recv().unwrap();
        assert!(matches!(response, ResponseMessage::Success));
    }

    #[tokio::test]
    async fn test_get_actor_sender_with_ping() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Ping the system to verify guardian is accessible
        ping_actor_system(&actor_system)
            .await
            .expect("Ping should succeed");

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_get_message_count_reference() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let count_ref = get_message_count_reference(&actor_system);

        // Verify we got a valid atomic reference
        let _count = count_ref.load(std::sync::atomic::Ordering::SeqCst);
    }

    #[tokio::test]
    async fn test_get_thread_count_reference() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let thread_ref = get_thread_count_reference(&actor_system);

        // Verify we got a valid atomic reference (value may vary due to system initialization)
        let _count = thread_ref.load(std::sync::atomic::Ordering::SeqCst);
        // Just verify we can read it without panicking
    }

    #[tokio::test]
    async fn test_get_total_messages() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let total = get_total_messages(&actor_system);

        // Should return a valid count (may be > 0 due to system initialization)
        assert!(total < 100); // Sanity check - shouldn't be huge on fresh system
    }

    #[tokio::test]
    async fn test_get_total_threads() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let total = get_total_threads(&actor_system);

        // Should return a valid count (may vary due to system initialization)
        assert!(total < 100); // Sanity check - shouldn't be huge on fresh system
    }

    #[tokio::test]
    async fn test_spawn_multiple_actors() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Spawn multiple actors using the API
        for i in 0..5 {
            let actor = TestActor {
                name: format!("Actor{}", i),
                counter: 0,
            };
            spawn_actor(&mut actor_system, actor, Some(format!("actor_{}", i))).await;
        }

        // Give actors time to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Verify system is still responsive via ping
        ping_actor_system(&actor_system)
            .await
            .expect("Ping should succeed");
    }

    #[tokio::test]
    async fn test_mixed_spawn_actors() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Spawn async actor
        let async_actor = TestActor {
            name: "AsyncActor".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, async_actor, Some("async".to_string())).await;

        // Spawn blocking actor
        let blocking_actor = BlockingTestActor {
            name: "BlockingActor".to_string(),
        };
        spawn_blocking_actor(
            &mut actor_system,
            blocking_actor,
            Some("blocking".to_string()),
        )
        .await;

        // Verify system is still responsive
        ping_actor_system(&actor_system)
            .await
            .expect("Ping should succeed");
    }

    #[tokio::test]
    async fn test_response_channels_with_messages() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let (tx, rx) = get_response_channel(&actor_system);

        // Send a successful response
        tx.send(ResponseMessage::Success).unwrap();

        // Wait for response with timeout
        let response = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx).await;

        assert!(response.is_ok());
    }

    #[test]
    fn test_sids_default_buffer_size() {
        // Verify the constant is set
        assert_eq!(SIDS_DEFAULT_BUFFER_SIZE, 100);
    }

    #[tokio::test]
    async fn test_multiple_responder_channels() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Create multiple response channels
        let (tx1, rx1) = get_response_channel(&actor_system);
        let (tx2, rx2) = get_response_channel(&actor_system);
        let (tx3, rx3) = get_response_channel(&actor_system);

        // Send responses
        tx1.send(ResponseMessage::Success).unwrap();
        tx2.send(ResponseMessage::Failure { message: None })
            .unwrap();
        tx3.send(ResponseMessage::Success).unwrap();

        // Verify all responses received
        assert!(rx1.await.is_ok());
        assert!(rx2.await.is_ok());
        assert!(rx3.await.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_blocking_channels() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Create multiple blocking response channels
        let (tx1, rx1) = get_blocking_response_channel(&actor_system);
        let (tx2, rx2) = get_blocking_response_channel(&actor_system);

        // Send responses
        tx1.send(ResponseMessage::Success).unwrap();
        tx2.send(ResponseMessage::Success).unwrap();

        // Verify all responses received
        assert!(rx1.recv().is_ok());
        assert!(rx2.recv().is_ok());
    }

    #[tokio::test]
    async fn test_list_actors() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Spawn actors with names
        let actor1 = TestActor {
            name: "Actor1".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, actor1, Some("FirstActor".to_string())).await;

        let actor2 = TestActor {
            name: "Actor2".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, actor2, Some("SecondActor".to_string())).await;

        // List all actors
        let actors = list_actors(&actor_system);

        // Should have 2 actors (IDs 0 and 1)
        assert_eq!(actors.len(), 2);
        assert_eq!(actors[0].0, 0); // First actor ID
        assert_eq!(actors[0].1, "FirstActor");
        assert_eq!(actors[1].0, 1); // Second actor ID
        assert_eq!(actors[1].1, "SecondActor");
    }

    #[tokio::test]
    async fn test_find_actor_by_name() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let actor = TestActor {
            name: "TestActor".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, actor, Some("NamedActor".to_string())).await;

        // Find actor by name
        let result = find_actor_by_name(&actor_system, "NamedActor");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Try to find non-existent actor
        let result = find_actor_by_name(&actor_system, "NonExistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_actor_exists() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let actor = TestActor {
            name: "TestActor".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, actor, None).await;

        // Check existing actor
        assert!(actor_exists(&actor_system, 0));

        // Check non-existent actor
        assert!(!actor_exists(&actor_system, 999));
    }

    #[tokio::test]
    async fn test_stop_actor() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        let actor = TestActor {
            name: "TestActor".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, actor, Some("StoppableActor".to_string())).await;

        // Verify actor exists
        assert!(actor_exists(&actor_system, 0));

        // Stop the actor
        let result = stop_actor(&mut actor_system, 0).await;
        assert!(result.is_ok());

        // Give actor time to stop
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Try to stop non-existent actor
        let result = stop_actor(&mut actor_system, 999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_actors_auto_names() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();

        // Spawn actors without names
        let actor1 = TestActor {
            name: "Actor1".to_string(),
            counter: 0,
        };
        spawn_actor(&mut actor_system, actor1, None).await;

        let actor2 = BlockingTestActor {
            name: "Actor2".to_string(),
        };
        spawn_blocking_actor(&mut actor_system, actor2, None).await;

        // List actors
        let actors = list_actors(&actor_system);

        // Should have auto-generated names
        assert_eq!(actors.len(), 2);
        assert!(actors[0].1.contains("Actor"));
        assert!(actors[1].1.contains("Actor"));
    }
}
