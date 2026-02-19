use super::messages::Message;
use log::info;
use tokio::sync::mpsc;


/// The main actor trait that all actors must implement.
#[trait_variant::make(Send)]
pub trait Actor<MType, Response>{
    async fn receive(&mut self, message: Message<MType, Response>) where Self: Sized + 'static;
}


/// Helper function for running an actor.
pub (super) async fn run_an_actor<MType, Response, T: Actor<MType, Response> + 'static>(mut actor : ActorImpl <T, MType, Response>) {
    while let Some(message) = actor.receiver.recv().await {
        if message.stop {
            break;
        }
        actor.receive(message).await;
    }

}

/// Helper function for running a blocking actor.
pub (super) async fn run_a_blocking_actor<MType, Response, T: Actor<MType, Response> + 'static>(mut actor: BlockingActorImpl<T, MType, Response>) {
    while let Ok(message) = actor.receiver.recv() {
        if message.stop {
            break;
        }
        actor.receive(message).await;
    }
}


/// Implements an actor with an Actor type T, and a ResponseMessage type Response.
/// 
/// A default ResponseMessage is available in actors::messages::ResponseMessage with basic functionality.
pub struct ActorImpl<T, MType, Response> {
    name: Option<String>,
    actor: T,
    receiver: mpsc::Receiver<Message<MType, Response>>,
}

impl <MType, Response, T: Actor<MType, Response> + 'static> ActorImpl <T, MType, Response> {
    pub async fn receive(&mut self, message: Message<MType, Response>){
        info!("Actor {} received message", self.name.clone().unwrap_or("Unnamed Actor".to_string()));
        T::receive(&mut self.actor, message).await;
    }
    pub fn new (name: Option<String>, actor: T, receiver: mpsc::Receiver::<Message<MType, Response>>) -> Self {

        ActorImpl {name, actor, receiver }
    }
}

/// Implementation of a blocking actor, which runs in a separate thread and uses std::sync::mpsc for communication.
/// 
/// A classic example of a blocking actor is one that will get data from http or other source.
pub struct BlockingActorImpl<T, MType, Response> {
    name: Option<String>,
    actor: T,
    receiver: std::sync::mpsc::Receiver<Message<MType, Response>>,
}

impl <MType, Response, T: Actor<MType, Response> + 'static> BlockingActorImpl<T, MType, Response> {

     pub async fn receive(&mut self, message: Message<MType, Response>){ 
        info!("Blocking actor {} received message", self.name.clone().unwrap_or("Unnamed Blocking Actor".to_string()));
        T::receive(&mut self.actor, message).await;
    }

    pub fn new (name: Option<String>, actor: T, receiver: std::sync::mpsc::Receiver<Message<MType, Response>>) -> BlockingActorImpl<T, MType, Response> {
        BlockingActorImpl { name, actor, receiver}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::messages::ResponseMessage;
    use tokio::sync::mpsc;
    use std::sync::{Arc, Mutex};

    struct TestPayload {
        _value: String,
    }

    struct CounterPayload {
        increment: i32,
    }
    struct EchoActor;

    impl Actor<TestPayload, ResponseMessage> for EchoActor {
        async fn receive(&mut self, message: Message<TestPayload, ResponseMessage>) {
            if let Some(_payload) = message.payload {
                if let Some(responder) = message.responder {
                    let _ = responder.send(ResponseMessage::Success);
                }
            }
        }
    }

    /// An actor that counts the messages it receives.
    struct CounterActor {
        count: Arc<Mutex<i32>>,
    }

    impl Actor<CounterPayload, ResponseMessage> for CounterActor {
        async fn receive(&mut self, message: Message<CounterPayload, ResponseMessage>) {
            if let Some(payload) = message.payload {
                let mut count = self.count.lock().unwrap();
                *count += payload.increment;
            }
        }
    }

    /// An actor that responds to blocking messages by sending a success response.
    struct BlockingEchoActor;

    impl Actor<TestPayload, ResponseMessage> for BlockingEchoActor {
        async fn receive(&mut self, message: Message<TestPayload, ResponseMessage>) {
            if let Some(blocking) = message.blocking {
                let _ = blocking.send(ResponseMessage::Success);
            }
        }
    }

    #[tokio::test]
    async fn test_actor_impl_creation() {
        let (_tx, rx) = mpsc::channel::<Message<TestPayload, ResponseMessage>>(1);
        let actor = EchoActor;
        let actor_impl = ActorImpl::new(Some("TestActor".to_string()), actor, rx);
        
        assert!(actor_impl.name.is_some());
        assert_eq!(actor_impl.name.unwrap(), "TestActor");
    }

    #[tokio::test]
    async fn test_actor_impl_creation_without_name() {
        let (_tx, rx) = mpsc::channel::<Message<TestPayload, ResponseMessage>>(1);
        let actor = EchoActor;
        let actor_impl = ActorImpl::new(None, actor, rx);
        
        assert!(actor_impl.name.is_none());
    }

    #[tokio::test]
    async fn test_actor_impl_receive() {
        let (_tx, rx) = mpsc::channel::<Message<TestPayload, ResponseMessage>>(1);
        let actor = EchoActor;
        let mut actor_impl = ActorImpl::new(Some("EchoActor".to_string()), actor, rx);
        
        let (responder_tx, responder_rx) = tokio::sync::oneshot::channel();
        let payload = TestPayload { _value: "test".to_string() };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: Some(responder_tx),
            blocking: None,
        };
        
        actor_impl.receive(message).await;
        
        let response = responder_rx.await.expect("Failed to receive response");
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_counter_actor() {
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        
        let (_tx, rx) = mpsc::channel::<Message<CounterPayload, ResponseMessage>>(10);
        let actor = CounterActor { count: count_clone };
        let mut actor_impl = ActorImpl::new(Some("CounterActor".to_string()), actor, rx);
        
        // Send multiple messages
        for i in 1..=5 {
            let payload = CounterPayload { increment: i };
            let message = Message {
                payload: Some(payload),
                stop: false,
                responder: None,
                blocking: None,
            };
            actor_impl.receive(message).await;
        }
        
        let final_count = *count.lock().unwrap();
        assert_eq!(final_count, 15); // 1+2+3+4+5 = 15
    }

    #[tokio::test]
    async fn test_run_an_actor_with_stop() {
        let (tx, rx) = mpsc::channel::<Message<CounterPayload, ResponseMessage>>(10);
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        
        let actor = CounterActor { count: count_clone };
        let actor_impl = ActorImpl::new(Some("CounterActor".to_string()), actor, rx);
        
        // Spawn the actor in a background task
        let actor_task = tokio::spawn(async move {
            run_an_actor(actor_impl).await;
        });
        
        // Send a regular message
        let payload = CounterPayload { increment: 5 };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: None,
            blocking: None,
        };
        tx.send(message).await.unwrap();
        
        // Give time for message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Verify the message was processed
        assert_eq!(*count.lock().unwrap(), 5);
        
        // Send a stop message
        let stop_message = Message {
            payload: None,
            stop: true,
            responder: None,
            blocking: None,
        };
        tx.send(stop_message).await.unwrap();
        
        // Wait for actor to finish
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            actor_task
        ).await;
        
        // The actor should have stopped cleanly
        assert!(result.is_ok(), "Actor should have stopped");
    }

    #[test]
    fn test_blocking_actor_impl_creation() {
        let (_tx, rx) = std::sync::mpsc::channel::<Message<TestPayload, ResponseMessage>>();
        let actor = BlockingEchoActor;
        let actor_impl = BlockingActorImpl::new(Some("BlockingTest".to_string()), actor, rx);
        
        assert!(actor_impl.name.is_some());
        assert_eq!(actor_impl.name.unwrap(), "BlockingTest");
    }

    #[test]
    fn test_blocking_actor_impl_creation_without_name() {
        let (_tx, rx) = std::sync::mpsc::channel::<Message<TestPayload, ResponseMessage>>();
        let actor = BlockingEchoActor;
        let actor_impl = BlockingActorImpl::new(None, actor, rx);
        
        assert!(actor_impl.name.is_none());
    }

    #[tokio::test]
    async fn test_blocking_actor_receive() {
        let (_tx, rx) = std::sync::mpsc::channel::<Message<TestPayload, ResponseMessage>>();
        let actor = BlockingEchoActor;
        let mut actor_impl = BlockingActorImpl::new(Some("BlockingEcho".to_string()), actor, rx);
        
        let (blocking_tx, blocking_rx) = std::sync::mpsc::sync_channel(1);
        let payload = TestPayload { _value: "test".to_string() };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: None,
            blocking: Some(blocking_tx),
        };
        
        actor_impl.receive(message).await;
        
        let response = blocking_rx.recv().expect("Failed to receive response");
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_run_an_actor_multiple_messages() {
        let (tx, rx) = mpsc::channel::<Message<CounterPayload, ResponseMessage>>(10);
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        
        let actor = CounterActor { count: count_clone };
        let actor_impl = ActorImpl::new(Some("CounterActor".to_string()), actor, rx);
        
        // Spawn the actor
        let actor_task = tokio::spawn(async move {
            run_an_actor(actor_impl).await;
        });
        
        // Send multiple messages
        for i in 1..=10 {
            let payload = CounterPayload { increment: i };
            let message = Message {
                payload: Some(payload),
                stop: false,
                responder: None,
                blocking: None,
            };
            tx.send(message).await.unwrap();
        }
        
        // Give time for messages to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Send stop message
        let stop_message = Message {
            payload: None,
            stop: true,
            responder: None,
            blocking: None,
        };
        tx.send(stop_message).await.unwrap();
        
        actor_task.await.unwrap();
        
        let final_count = *count.lock().unwrap();
        assert_eq!(final_count, 55); // Sum of 1 to 10
    }

    #[tokio::test]
    async fn test_actor_stops_on_stop_message() {
        let (tx, rx) = mpsc::channel::<Message<TestPayload, ResponseMessage>>(10);
        let actor = EchoActor;
        let actor_impl = ActorImpl::new(Some("EchoActor".to_string()), actor, rx);
        
        let actor_task = tokio::spawn(async move {
            run_an_actor(actor_impl).await;
        });
        
        // Send stop message immediately
        let stop_message = Message {
            payload: None,
            stop: true,
            responder: None,
            blocking: None,
        };
        tx.send(stop_message).await.unwrap();
        
        // Actor should complete quickly
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            actor_task
        ).await;
        
        assert!(result.is_ok(), "Actor should have stopped");
    }

    #[test]
    fn test_blocking_actor_stops_on_stop_message() {
        let (tx, rx) = std::sync::mpsc::channel::<Message<TestPayload, ResponseMessage>>();
        let actor = BlockingEchoActor;
        let actor_impl = BlockingActorImpl::new(Some("BlockingEcho".to_string()), actor, rx);
        
        // Spawn blocking actor in a thread
        let handle = std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                run_a_blocking_actor(actor_impl).await;
            });
        });
        
        // Send stop message
        let stop_message = Message {
            payload: None,
            stop: true,
            responder: None,
            blocking: None,
        };
        tx.send(stop_message).unwrap();
        
        // Wait for thread to complete
        handle.join().unwrap();
    }
}