use std::{collections::HashMap, sync::atomic::AtomicUsize, time::Duration};

use crate::actors::actor::{ActorImpl, BlockingActorImpl};
use crate::config::SidsConfig;

use super::{
    actor::Actor,
    actor_ref::{ActorRef, BlockingActorRef},
    channel_factory::ChannelFactory,
    messages::{Message, ResponseMessage},
};
use log::{info, warn};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::timeout;

#[cfg(feature = "visualize")]
use crate::supervision::{ActorMetrics, SupervisionData, SupervisionSummary};

// Actor name constants for logging
const GUARDIAN_ACTOR_NAME: &str = "GUARDIAN";
const ACTOR_SYSTEM_NAME: &str = "Actor System";

struct Guardian {
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl<MType: Send + Clone> Actor<MType, ResponseMessage> for Guardian {
    async fn receive(&mut self, message: Message<MType, ResponseMessage>) {
        info!(actor=GUARDIAN_ACTOR_NAME; "Guardian received a message");
        if message.stop {
            info!(actor=GUARDIAN_ACTOR_NAME; "Guardian received a stop message. Shutting down...");
            if let Some(ref shutdown_tx) = self.shutdown_tx {
                let _ = shutdown_tx.send(());
                info!(actor=GUARDIAN_ACTOR_NAME; "Shutdown signal broadcast to all actors");
            }
        }
        match message.responder {
            Some(responder) => {
                responder
                    .send(ResponseMessage::Success)
                    .expect("Failed to send response");
            }
            None => {
                info!("No responder found");
            }
        }
    }
}


/// The ActorSystem is the main entry point for the actor system. It is responsible for creating the guardian actor and sending messages to the guardian actor.
///
/// The ActorSystem is designed to be an actor reference for the guardian actor that manages all other actors in the system.
/// In practice, it is the only actor_reference that is directly interacted with by the user.
///
/// # Example
/// ```rust
/// use sids::actors;
/// use sids::actors::messages::{Message, ResponseMessage};
/// use sids::actors::actor::Actor;
///
/// /// Sample actor type to receive message.
///
/// struct SampleActor;
/// impl Actor<String, ResponseMessage> for SampleActor {
///    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
///       message.responder.unwrap().send(ResponseMessage::Success).expect("Failed to send response");
///    }
/// }
///
/// pub async fn run_system() {
///    let actor = SampleActor;
///    
///    // Creates a new actor system that uses String as the message type.
///    let mut actor_system = actors::start_actor_system::<String, ResponseMessage>();
///    let (tx, rx) = actors::get_response_channel(&mut actor_system);
///    let message = Message { payload: Some("My String Message".to_string()), stop: false, responder: Some(tx), blocking: None };
///    actors::spawn_actor(&mut actor_system, actor, Some("Sample Actor".to_string())).await;
///    actors::send_message_by_id(&mut actor_system, 1, message).await;
///    let response = rx.await.expect("Failed to receive response");
///    assert_eq!(response, ResponseMessage::Success);
/// }
///
/// ```
pub struct ActorSystem<MType: Send + Clone + 'static, Response: Send + Clone + 'static> {
    _guardian: ActorRef<MType, ResponseMessage>,
    actors: HashMap<u32, ActorRef<MType, Response>>,
    blocking_actors: HashMap<u32, BlockingActorRef<MType, Response>>,
    total_messages: &'static AtomicUsize,
    total_threads: &'static AtomicUsize,
    snd: mpsc::Sender<Message<MType, ResponseMessage>>,
    shutdown_tx: broadcast::Sender<()>,
    actor_buffer_size: usize,
    shutdown_timeout_ms: Option<u64>,
    #[cfg(feature = "visualize")]
    supervision: SupervisionData,
}

impl<MType: Send + Clone + 'static, Response: Send + Clone + 'static> ChannelFactory<MType, Response> for ActorSystem<MType, Response> {
    fn create_actor_channel(
        &self,
    ) -> (
        tokio::sync::mpsc::Sender<Message<MType, Response>>,
        tokio::sync::mpsc::Receiver<Message<MType, Response>>,
    ) {
        mpsc::channel::<Message<MType, Response>>(self.actor_buffer_size)
    }

    fn create_blocking_actor_channel(
        &self,
    ) -> (
        std::sync::mpsc::Sender<Message<MType, Response>>,
        std::sync::mpsc::Receiver<Message<MType, Response>>,
    ) {
        std::sync::mpsc::channel::<Message<MType, Response>>()
    }

    fn create_response_channel(
        &self,
    ) -> (
        tokio::sync::oneshot::Sender<Response>,
        tokio::sync::oneshot::Receiver<Response>,
    ) {
        oneshot::channel::<Response>()
    }

    fn create_blocking_response_channel(
        &self,
    ) -> (
        std::sync::mpsc::SyncSender<Response>,
        std::sync::mpsc::Receiver<Response>,
    ) {
        std::sync::mpsc::sync_channel::<Response>(1)
    }
}

impl<MType: Send + Clone + 'static, Response: Send + Clone + 'static> ActorSystem<MType, Response> {
    /// Create a new ActorSystem
    ///
    /// The ActorSystem will start by launching a guardian, which is a non-blocking officer-actor that manages all other actors in the system.
    /// The guardian will be dormant until start_system is called in the ActorSystem.
    pub(super) fn new() -> Self {
        Self::new_with_config(SidsConfig::default())
    }

    pub(super) fn new_with_config(config: SidsConfig) -> Self {
        let actor_buffer_size = config.actor_system.actor_buffer_size;
        let shutdown_timeout_ms = config.actor_system.shutdown_timeout_ms;
        let (tx, rx) = mpsc::channel::<Message<MType, ResponseMessage>>(actor_buffer_size);
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        info!(actor = "GUARDIAN"; "Guardian channel and actor created. Launching...");
        info!(actor = "GUARDIAN"; "Guardian actor spawned");
        let guardian = ActorImpl::new(Some("Guardian Type".to_string()), Guardian { shutdown_tx: Some(shutdown_tx.clone()) }, rx, None);
        info!(actor = "GUARDIAN"; "Actor system created");
        static MESSAGE_MONITOR: AtomicUsize = AtomicUsize::new(0);
        static THREAD_MONITOR: AtomicUsize = AtomicUsize::new(0);
        let actor_ref = ActorRef::new(guardian, tx.clone(), &THREAD_MONITOR, &MESSAGE_MONITOR);
        let actors = HashMap::<u32, ActorRef<MType, Response>>::new();
        let blocking_actors = HashMap::new();
        ActorSystem {
            _guardian: actor_ref,
            actors,
            blocking_actors,
            total_messages: &MESSAGE_MONITOR,
            total_threads: &THREAD_MONITOR,
            snd: tx,
            shutdown_tx,
            actor_buffer_size,
            shutdown_timeout_ms,
            #[cfg(feature = "visualize")]
            supervision: SupervisionData::new(),
        }
    }

    pub async fn spawn_actor<T>(&mut self, actor: T, name: Option<String>)
    where
        T: Actor<MType, Response> + 'static,
    {
        info!(actor=ACTOR_SYSTEM_NAME;"Spawning actor within the actor system.");
        let (snd, rec) = self.create_actor_channel();
        let shutdown_rx = self.shutdown_tx.subscribe();
        let actor_impl = ActorImpl::new(name.clone(), actor, rec, Some(shutdown_rx));
        let actor_ref = ActorRef::new(actor_impl, snd, self.total_threads, self.total_messages);
        let actor_id = self.actors.len() as u32;
        self.actors.insert(actor_id, actor_ref);
        
        #[cfg(feature = "visualize")]
        {
            self.record_actor_spawn(actor_id, name.unwrap_or_else(|| format!("Actor<{}>", actor_id)));
        }
        
        info!(actor=ACTOR_SYSTEM_NAME; "Actor spawned successfully with id: {}", actor_id);
    }

    pub(super) fn spawn_blocking_actor<T>(&mut self, actor: T, name: Option<String>)
    where
        T: Actor<MType, Response> + 'static,
    {
        info!(actor=ACTOR_SYSTEM_NAME; "Spawning blocking actor within the actor system.");
        let (snd, rec) = self.create_blocking_actor_channel();
        let shutdown_rx = self.shutdown_tx.subscribe();
        let actor_impl = BlockingActorImpl::new(name.clone(), actor, rec, Some(shutdown_rx));
        let actor_ref = BlockingActorRef::new(actor_impl, snd, self.total_threads, self.total_messages);
        let actor_id = self.blocking_actors.len() as u32;
        self.blocking_actors.insert(actor_id, actor_ref);
        
        #[cfg(feature = "visualize")]
        {
            self.record_actor_spawn(actor_id, name.unwrap_or_else(|| format!("BlockingActor<{}>", actor_id)));
        }
        
        info!(actor=ACTOR_SYSTEM_NAME; "Blocking actor spawned successfully with id: {}", actor_id);
    }

    pub(super) async fn send_message_to_actor(&mut self, actor_id: u32, message: Message<MType, Response>) {
        if let Message {
            payload: _,
            stop: _,
            responder: None,
            blocking: Some(_),
        } = &message
        {
            let blocking_actor = self
                .blocking_actors
                .get_mut(&actor_id)
                .expect("Failed to get blocking actor");
            blocking_actor.send(message);
            
            #[cfg(feature = "visualize")]
            {
                self.record_message_processed(actor_id);
            }
        } else if let Message {
            payload: _,
            stop: _,
            responder: _,
            blocking: None,
        } = &message
        {
            let actor = self.actors.get_mut(&actor_id).expect("Failed to get actor");
            actor.send(message).await;
            
            #[cfg(feature = "visualize")]
            {
                self.record_message_processed(actor_id);
            }
        } else {
            warn!("No actor found with id: {}", actor_id);
        }
    }

    pub(super) async fn ping_system(&self) {
        info!("Pinging system");
        self.snd
            .send(Message {
                payload: None,
                stop: false,
                responder: None,
                blocking: None,
            })
            .await
            .expect("Failed to send message");
    }
    /// Send a shutdown signal to the guardian, which will broadcast shutdown to all actors.
    pub async fn shutdown(&self) -> Result<(), String> {
        info!("Sending shutdown signal to guardian");
        let (tx, rx) = self.create_guardian_response_channel();
        self.snd
            .send(Message {
                payload: None,
                stop: true,
                responder: Some(tx),
                blocking: None,
            })
            .await
            .map_err(|e| format!("Failed to send shutdown message: {}", e))?;
        
        if let Some(timeout_ms) = self.shutdown_timeout_ms {
            match timeout(Duration::from_millis(timeout_ms), rx).await {
                Ok(result) => result
                    .map(|_| {
                        info!("Guardian confirmed shutdown");
                    })
                    .map_err(|e| format!("Guardian failed to respond: {}", e)),
                Err(_) => Err(format!("Guardian shutdown timed out after {} ms", timeout_ms)),
            }
        } else {
            rx.await
                .map(|_| {
                    info!("Guardian confirmed shutdown");
                })
                .map_err(|e| format!("Guardian failed to respond: {}", e))
        }
    }

    /// Get a receiver for the shutdown broadcast signal.
    /// Useful if you want to await system shutdown outside the actor system.
    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    fn create_guardian_response_channel(
        &self,
    ) -> (
        tokio::sync::oneshot::Sender<ResponseMessage>,
        tokio::sync::oneshot::Receiver<ResponseMessage>,
    ) {
        oneshot::channel::<ResponseMessage>()
    }
    pub fn get_actor_ref(&self, id: u32) -> ActorRef<MType, Response> {
        self.actors.get(&id).expect("Failed to get actor").clone()
    }

    pub fn get_actor_count(&self) -> usize {
        self.actors.len()
    }

    pub fn get_thread_count_reference(&self) -> &'static AtomicUsize {
        self.total_messages
    }

    pub fn get_message_count_reference(&self) -> &'static AtomicUsize {
        self.total_threads
    }

    pub fn get_thread_count(&self) -> usize {
        self.total_threads.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_message_count(&self) -> usize {
        self.total_messages.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(feature = "visualize")]
    pub fn record_actor_spawn(&mut self, actor_id: u32, actor_type: String) {
        let actor_id_str = format!("actor-{}", actor_id);
        let actor_metrics = ActorMetrics::new(actor_id_str, actor_type);
        self.supervision.actors.insert(format!("actor-{}", actor_id), actor_metrics);
    }

    #[cfg(feature = "visualize")]
    pub fn record_message_processed(&mut self, actor_id: u32) {
        self.supervision.record_message_processed(&format!("actor-{}", actor_id));
    }

    #[cfg(feature = "visualize")]
    pub fn record_message_sent(&mut self, from_id: u32, to_id: u32) {
        let from = format!("actor-{}", from_id);
        let to = format!("actor-{}", to_id);
        if let Some(actor) = self.supervision.actors.get_mut(&from) {
            actor.record_message_sent(to);
        }
    }

    #[cfg(feature = "visualize")]
    pub fn get_supervision_data(&self) -> SupervisionData {
        self.supervision.clone()
    }

    #[cfg(feature = "visualize")]
    pub fn get_supervision_summary(&self) -> SupervisionSummary {
        self.supervision.summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::messages::ResponseMessage;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::time::{timeout, Duration};

    // Test payload types
    #[derive(Clone)]
    struct StringPayload {
        _content: String,
    }

    #[derive(Clone)]
    struct CounterPayload {
        value: i32,
    }

    // Simple echo actor that responds back
    struct EchoActor;

    impl Actor<StringPayload, ResponseMessage> for EchoActor {
        async fn receive(&mut self, message: Message<StringPayload, ResponseMessage>) {
            if let Some(responder) = message.responder {
                let _ = responder.send(ResponseMessage::Success);
            }
        }
    }

    // Actor that accumulates values
    struct AccumulatorActor {
        total: Arc<Mutex<i32>>,
    }

    impl Actor<CounterPayload, ResponseMessage> for AccumulatorActor {
        async fn receive(&mut self, message: Message<CounterPayload, ResponseMessage>) {
            if let Some(payload) = message.payload {
                let mut total = self.total.lock().unwrap();
                *total += payload.value;
            }
        }
    }

    // Blocking echo actor
    struct BlockingEchoActor;

    impl Actor<StringPayload, ResponseMessage> for BlockingEchoActor {
        async fn receive(&mut self, message: Message<StringPayload, ResponseMessage>) {
            if let Some(blocking) = message.blocking {
                let _ = blocking.send(ResponseMessage::Success);
            }
        }
    }

    struct CountingActor {
        counter: Arc<AtomicUsize>,
    }

    impl Actor<StringPayload, ResponseMessage> for CountingActor {
        async fn receive(&mut self, message: Message<StringPayload, ResponseMessage>) {
            if message.payload.is_some() {
                self.counter.fetch_add(1, Ordering::SeqCst);
                if let Some(responder) = message.responder {
                    let _ = responder.send(ResponseMessage::Success);
                }
            }
        }
    }

    struct SilentActor;

    impl Actor<StringPayload, ResponseMessage> for SilentActor {
        async fn receive(&mut self, _message: Message<StringPayload, ResponseMessage>) {
            // Simulate a failure to respond without panicking the runtime.
        }
    }

    #[tokio::test]
    async fn test_actor_system_creation() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        assert_eq!(actor_system.actors.len(), 0);
        assert_eq!(actor_system.blocking_actors.len(), 0);
    }

    #[tokio::test]
    async fn test_spawn_actor() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = EchoActor;
        
        actor_system.spawn_actor(actor, Some("TestActor".to_string())).await;
        
        assert_eq!(actor_system.actors.len(), 1);
        assert_eq!(actor_system.get_actor_count(), 1);
    }

    #[tokio::test]
    async fn test_spawn_multiple_actors() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        
        for i in 0..5 {
            let actor = EchoActor;
            actor_system.spawn_actor(actor, Some(format!("Actor_{}", i))).await;
        }
        
        assert_eq!(actor_system.actors.len(), 5);
        assert_eq!(actor_system.get_actor_count(), 5);
    }

    #[tokio::test]
    async fn test_spawn_blocking_actor() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = BlockingEchoActor;
        
        actor_system.spawn_blocking_actor(actor, Some("BlockingActor".to_string()));
        
        assert_eq!(actor_system.blocking_actors.len(), 1);
    }

    #[tokio::test]
    async fn test_get_actor_ref() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = EchoActor;
        
        actor_system.spawn_actor(actor, Some("TestActor".to_string())).await;
        
        let actor_ref = actor_system.get_actor_ref(0);
        assert!(actor_ref.sender.capacity() > 0);
    }

    #[tokio::test]
    async fn test_send_message_to_actor() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = EchoActor;
        
        actor_system.spawn_actor(actor, Some("EchoActor".to_string())).await;
        
        let (tx, rx) = actor_system.create_response_channel();
        let payload = StringPayload { _content: "test".to_string() };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: Some(tx),
            blocking: None,
        };
        
        actor_system.send_message_to_actor(0, message).await;
        
        let response = rx.await.expect("Failed to receive response");
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_send_message_to_blocking_actor() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = BlockingEchoActor;
        
        actor_system.spawn_blocking_actor(actor, Some("BlockingEcho".to_string()));
        
        let (tx, rx) = actor_system.create_blocking_response_channel();
        let payload = StringPayload { _content: "test".to_string() };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: None,
            blocking: Some(tx),
        };
        
        actor_system.send_message_to_actor(0, message).await;
        
        // Give time for blocking actor to process
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let response = rx.recv().expect("Failed to receive response");
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_actor_accumulator() {
        let mut actor_system = ActorSystem::<CounterPayload, ResponseMessage>::new();
        let total = Arc::new(Mutex::new(0));
        let total_clone = total.clone();
        
        let actor = AccumulatorActor { total: total_clone };
        actor_system.spawn_actor(actor, Some("Accumulator".to_string())).await;
        
        // Send multiple messages
        for i in 1..=10 {
            let payload = CounterPayload { value: i };
            let message = Message {
                payload: Some(payload),
                stop: false,
                responder: None,
                blocking: None,
            };
            actor_system.send_message_to_actor(0, message).await;
        }
        
        // Give actors time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let result = *total.lock().unwrap();
        assert_eq!(result, 55); // Sum of 1 to 10
    }

    #[tokio::test]
    async fn test_ping_system() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        
        // Should not panic
        actor_system.ping_system().await;
    }

    #[tokio::test]
    async fn test_get_actor_count() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        
        assert_eq!(actor_system.get_actor_count(), 0);
        
        actor_system.spawn_actor(EchoActor, Some("Actor1".to_string())).await;
        assert_eq!(actor_system.get_actor_count(), 1);
        
        actor_system.spawn_actor(EchoActor, Some("Actor2".to_string())).await;
        assert_eq!(actor_system.get_actor_count(), 2);
    }

    #[tokio::test]
    async fn test_create_actor_channel() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        let (tx, _rx) = actor_system.create_actor_channel();
        
        assert!(tx.capacity() > 0);
    }

    #[tokio::test]
    async fn test_create_blocking_actor_channel() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        let (tx, _rx) = actor_system.create_blocking_actor_channel();
        
        // Should successfully create channels
        let test_msg = Message {
            payload: Some("test".to_string()),
            stop: false,
            responder: None,
            blocking: None,
        };
        assert!(tx.send(test_msg).is_ok());
    }

    #[tokio::test]
    async fn test_create_response_channel() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        let (tx, rx) = actor_system.create_response_channel();
        
        tx.send(ResponseMessage::Success).unwrap();
        let response = rx.await.unwrap();
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_create_blocking_response_channel() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        let (tx, rx) = actor_system.create_blocking_response_channel();
        
        tx.send(ResponseMessage::Success).unwrap();
        let response = rx.recv().unwrap();
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_guardian_receives_message() {
        let actor_system = ActorSystem::<String, ResponseMessage>::new();
        
        let (tx, rx) = tokio::sync::oneshot::channel();
        let message = Message {
            payload: Some("test".to_string()),
            stop: false,
            responder: Some(tx),
            blocking: None,
        };
        
        actor_system.snd.send(message).await.unwrap();
        
        let response = rx.await.expect("Guardian should respond");
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_multiple_actors_concurrent_messages() {
        let mut actor_system = ActorSystem::<CounterPayload, ResponseMessage>::new();
        
        // Create 3 accumulator actors
        let totals: Vec<Arc<Mutex<i32>>> = (0..3)
            .map(|_| Arc::new(Mutex::new(0)))
            .collect();
        
        for total in &totals {
            let actor = AccumulatorActor { total: total.clone() };
            actor_system.spawn_actor(actor, None).await;
        }
        
        // Send messages to each actor
        for actor_id in 0..3 {
            for value in 1..=5 {
                let payload = CounterPayload { value };
                let message = Message {
                    payload: Some(payload),
                    stop: false,
                    responder: None,
                    blocking: None,
                };
                actor_system.send_message_to_actor(actor_id, message).await;
            }
        }
        
        // Give time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Each actor should have sum of 1 to 5 = 15
        for total in totals {
            assert_eq!(*total.lock().unwrap(), 15);
        }
    }

    #[tokio::test]
    async fn test_actor_system_with_different_types() {
        // Test that we can create actor systems with different message types
        let _system1 = ActorSystem::<String, ResponseMessage>::new();
        let _system2 = ActorSystem::<i32, ResponseMessage>::new();
        let _system3 = ActorSystem::<Vec<u8>, ResponseMessage>::new();
        
        // Just verify they compile and construct
    }

    #[tokio::test]
    async fn test_spawn_actor_without_name() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = EchoActor;
        
        actor_system.spawn_actor(actor, None).await;
        
        assert_eq!(actor_system.get_actor_count(), 1);
    }

    #[tokio::test]
    async fn test_spawn_blocking_actor_without_name() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let actor = BlockingEchoActor;
        
        actor_system.spawn_blocking_actor(actor, None);
        
        assert_eq!(actor_system.blocking_actors.len(), 1);
    }

    #[tokio::test]
    async fn test_shutdown_signal_broadcast() {
        // Test that the shutdown broadcast channel can be subscribed to
        let actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        
        // Subscribe to the shutdown signal (does not receive yet)
        let mut shutdown_rx1 = actor_system.subscribe_shutdown();
        let mut shutdown_rx2 = actor_system.subscribe_shutdown();
        
        // Manually broadcast shutdown signal
        let _ = actor_system.shutdown_tx.send(());
        
        // Both receivers should get the signal
        let result1 = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            shutdown_rx1.recv()
        ).await;
        let result2 = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            shutdown_rx2.recv()
        ).await;
        
        assert!(result1.is_ok(), "First receiver should get shutdown signal");
        assert!(result2.is_ok(), "Second receiver should get shutdown signal");
    }

    #[tokio::test]
    async fn test_actor_stop_prevents_future_messages() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let actor = CountingActor {
            counter: counter.clone(),
        };

        actor_system.spawn_actor(actor, Some("CountingActor".to_string())).await;

        let (tx, rx) = actor_system.create_response_channel();
        let message = Message {
            payload: Some(StringPayload {
                _content: "first".to_string(),
            }),
            stop: false,
            responder: Some(tx),
            blocking: None,
        };
        actor_system.send_message_to_actor(0, message).await;
        let response = rx.await.expect("Failed to receive response");
        assert_eq!(response, ResponseMessage::Success);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        let stop_message = Message {
            payload: None,
            stop: true,
            responder: None,
            blocking: None,
        };
        actor_system.send_message_to_actor(0, stop_message).await;
        tokio::time::sleep(Duration::from_millis(25)).await;

        let (tx2, rx2) = actor_system.create_response_channel();
        let message_after_stop = Message {
            payload: Some(StringPayload {
                _content: "after".to_string(),
            }),
            stop: false,
            responder: Some(tx2),
            blocking: None,
        };
        actor_system.send_message_to_actor(0, message_after_stop).await;

        let result = timeout(Duration::from_millis(50), rx2).await;
        match result {
            Err(_) => {}
            Ok(Err(_)) => {}
            Ok(Ok(_)) => panic!("Actor should not respond after stop"),
        }
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_actor_panic_does_not_break_other_actors() {
        let mut actor_system = ActorSystem::<StringPayload, ResponseMessage>::new();

        actor_system
            .spawn_actor(SilentActor, Some("SilentActor".to_string()))
            .await;
        actor_system
            .spawn_actor(EchoActor, Some("EchoActor".to_string()))
            .await;

        let (tx, rx) = actor_system.create_response_channel();
        let panic_message = Message {
            payload: Some(StringPayload {
                _content: "boom".to_string(),
            }),
            stop: false,
            responder: Some(tx),
            blocking: None,
        };
        actor_system.send_message_to_actor(0, panic_message).await;

        let silent_result = timeout(Duration::from_millis(50), rx).await;
        match silent_result {
            Err(_) => {}
            Ok(Err(_)) => {}
            Ok(Ok(_)) => panic!("Silent actor should not respond"),
        }

        let (tx2, rx2) = actor_system.create_response_channel();
        let echo_message = Message {
            payload: Some(StringPayload {
                _content: "ok".to_string(),
            }),
            stop: false,
            responder: Some(tx2),
            blocking: None,
        };
        actor_system.send_message_to_actor(1, echo_message).await;

        let echo_response = timeout(Duration::from_millis(50), rx2)
            .await
            .expect("Echo actor should respond")
            .expect("Response channel should be open");
        assert_eq!(echo_response, ResponseMessage::Success);
    }
}

