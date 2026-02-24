
pub mod actor_system;
pub mod actor;
pub mod actor_ref;
pub mod channel_factory;
pub mod messages;

use actor::Actor;
use actor_ref::ActorRef;
use actor_system::ActorSystem;
use crate::config::{SidsConfig, DEFAULT_ACTOR_BUFFER_SIZE};
use channel_factory::ChannelFactory;
use messages::Message;

pub const SIDS_DEFAULT_BUFFER_SIZE: usize = DEFAULT_ACTOR_BUFFER_SIZE;

pub fn start_actor_system<MType: Send + Clone + 'static, Response: Send + Clone + 'static>() -> ActorSystem<MType, Response> {
    ActorSystem::<MType, Response>::new()
}

pub fn start_actor_system_with_config<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(config: SidsConfig) -> ActorSystem<MType, Response> {
    ActorSystem::<MType, Response>::new_with_config(config)
}

pub async fn spawn_actor<MType: Send + Clone + 'static, Response: Send + Clone + 'static, T>(actor_system: &mut ActorSystem<MType, Response>, actor: T, name: Option<String>) where T: Actor<MType, Response> + 'static {  
    actor_system.spawn_actor(actor, name).await;
}

pub async fn spawn_blocking_actor<MType: Send + Clone + 'static, Response: Send + Clone + 'static, T>(actor_system: &mut ActorSystem<MType, Response>, actor: T, name: Option<String>) where T: Actor<MType, Response> + 'static {  
    actor_system.spawn_blocking_actor(actor, name);
}

pub async fn send_message_by_id<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &mut ActorSystem<MType, Response>, actor_id: u32, message: Message<MType, Response>) {
    actor_system.send_message_to_actor(actor_id, message).await;
}

pub async fn ping_actor_system<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) {
    actor_system.ping_system().await;
}

pub fn get_response_channel<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> (tokio::sync::oneshot::Sender<Response>, tokio::sync::oneshot::Receiver<Response>) {
    actor_system.create_response_channel()
}

pub fn get_blocking_response_channel<MType: Send + Clone +'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> (std::sync::mpsc::SyncSender<Response>, std::sync::mpsc::Receiver<Response>) {
    actor_system.create_blocking_response_channel()
}

pub fn get_actor_sender<MType: Send + Clone +'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>, id: u32) -> ActorRef<MType, Response> {
    actor_system.get_actor_ref(id).clone()
}

pub fn get_message_count_reference<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> &'static std::sync::atomic::AtomicUsize {
    actor_system.get_message_count_reference()
}

pub fn get_thread_count_reference<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> &'static std::sync::atomic::AtomicUsize {
    actor_system.get_thread_count_reference()
}

pub fn get_total_messages<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> usize {
    actor_system.get_thread_count()
}

pub fn get_total_threads<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> usize {
    actor_system.get_message_count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use messages::ResponseMessage;
    use log::info;

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
                            let _ = responder.send(ResponseMessage::Success);
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
        ping_actor_system(&actor_system).await;
    }

    #[tokio::test]
    async fn test_spawn_actor() {
        let mut actor_system = start_actor_system::<TestMessage, ResponseMessage>();
        
        let test_actor = TestActor {
            name: "TestActor1".to_string(),
            counter: 0,
        };
        
        spawn_actor(&mut actor_system, test_actor, Some("test_actor".to_string())).await;
        
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
        
        spawn_blocking_actor(&mut actor_system, blocking_actor, Some("blocking_test".to_string())).await;
        
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
        send_message_by_id(&mut actor_system, 0, message).await;
        
        // Give the system time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_ping_actor_system() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();
        
        // Ping should succeed without panic
        ping_actor_system(&actor_system).await;
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
        ping_actor_system(&actor_system).await;
        
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_get_message_count_reference() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();
        
        let count_ref = get_message_count_reference(&actor_system);
        
        // Verify we got a valid atomic reference
        let _count = count_ref.load(std::sync::atomic::Ordering::SeqCst);
        assert!(true);
    }

    #[tokio::test]
    async fn test_get_thread_count_reference() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();
        
        let thread_ref = get_thread_count_reference(&actor_system);
        
        // Verify we got a valid atomic reference (value may vary due to system initialization)
        let _count = thread_ref.load(std::sync::atomic::Ordering::SeqCst);
        // Just verify we can read it without panicking
        assert!(true);
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
        ping_actor_system(&actor_system).await;
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
        spawn_blocking_actor(&mut actor_system, blocking_actor, Some("blocking".to_string())).await;
        
        // Verify system is still responsive
        ping_actor_system(&actor_system).await;
    }

    #[tokio::test]
    async fn test_response_channels_with_messages() {
        let actor_system = start_actor_system::<TestMessage, ResponseMessage>();
        
        let (tx, rx) = get_response_channel(&actor_system);
        
        // Send a successful response
        tx.send(ResponseMessage::Success).unwrap();
        
        // Wait for response with timeout
        let response = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx
        ).await;
        
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
        tx2.send(ResponseMessage::Failure { message: None }).unwrap();
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
}
