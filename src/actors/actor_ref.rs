use std::sync::atomic::AtomicUsize;

use super::actor::{run_an_actor, Actor, ActorImpl, BlockingActorImpl};
use super::messages::Message;
use crate::actors::actor::run_a_blocking_actor;
use log::info;

// Actor name constants for logging
const TOKIO_ACTOR_NAME: &str = "Tokio Actor";
const BLOCKING_ACTOR_NAME: &str = "Blocking Actor";

#[derive(Debug, Clone)]

/// An actor reference initiates a thread and provides a channel for sending messages to the actor.
///
/// # Example
/// ```rust
/// use sids::actors::{actor_system::ActorSystem, actor_ref::ActorRef, actor::{Actor, ActorImpl}, messages::{Message, ResponseMessage}, spawn_actor, start_actor_system};
/// use sids::actors::channel_factory::ChannelFactory;
/// use std::sync::atomic::AtomicUsize;
/// use env_logger::{Builder, Env};
/// fn get_loggings() {
///     let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
///    Builder::from_env(env).init()
/// }
/// #[derive(Debug, Clone)]
/// enum ChatMessage {
///   Hello { name: String },
///  Goodbye,
/// StringMessage(String),
/// }
/// struct SampleActor;
/// impl Actor<ChatMessage, ResponseMessage> for SampleActor {
///   async fn receive(&mut self, message: Message<ChatMessage, ResponseMessage>) {
///      println!("Received message: {:?}", message.payload);
///  }
/// }
/// #[tokio::main]
/// async fn main() {
/// static THREAD_MONITOR: AtomicUsize = AtomicUsize::new(0);
/// static SEND_MONITOR: AtomicUsize = AtomicUsize::new(0);
/// get_loggings();
/// let mut actor_system = start_actor_system::<ChatMessage, ResponseMessage>();
/// let (tx, _rx) = tokio::sync::mpsc::channel::<Message<ChatMessage, ResponseMessage>>(100);
/// let actor = ActorImpl::new(None, SampleActor, _rx, None);
/// let actor_ref = ActorRef::new(actor, tx, &SEND_MONITOR, &THREAD_MONITOR);
/// let message = Message {payload: Some(ChatMessage::Hello { name: "Alice".to_string() }), stop: false, responder: None, blocking: None};
/// actor_ref.send(message).await;
/// }
/// ```
pub struct ActorRef<MType, Response>
where
    MType: Send + 'static,
    Response: Send + 'static,
{
    send_monitor: &'static AtomicUsize,
    pub sender: tokio::sync::mpsc::Sender<Message<MType, Response>>,
}

impl<MType: Send, Response: Send> ActorRef<MType, Response> {
    pub fn new<T: Actor<MType, Response> + 'static>(
        actor: ActorImpl<T, MType, Response>,
        sender: tokio::sync::mpsc::Sender<Message<MType, Response>>,
        thread_monitor: &'static AtomicUsize,
        send_monitor: &'static AtomicUsize,
    ) -> Self {
        info!(actor = TOKIO_ACTOR_NAME; "Spawning a Tokio Actor.");
        tokio::spawn(async move {
            thread_monitor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            run_an_actor::<MType, Response, T>(actor).await;
        });
        info!(actor = TOKIO_ACTOR_NAME; "Actor spawned successfully.");
        ActorRef {
            send_monitor,
            sender,
        }
    }

    pub async fn send(&self, message: Message<MType, Response>) {
        info!(actor = TOKIO_ACTOR_NAME; "Sending message to Tokio Actor from the ActorRef.");
        let _ = self.sender.send(message).await;
        self.send_monitor
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Debug)]
/// An actor reference that requires blocking such as for retrieving data from an http request.
///
/// # Example
/// ```rust
/// use sids::actors::{actor_ref::BlockingActorRef, actor::{Actor, BlockingActorImpl}, messages::ResponseMessage, get_blocking_response_channel, spawn_blocking_actor, start_actor_system};
/// use sids::actors::messages::Message;
/// use std::sync::atomic::AtomicUsize;
/// use env_logger::{Builder, Env};
/// fn get_loggings() {
///    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
///   Builder::from_env(env).init()
/// }
/// #[derive(Debug, Clone)]
/// enum BlockingChatMessage {
/// Hello { name: String },
/// Goodbye,
/// StringMessage(String),
/// }
/// struct SampleBlockingActor;
/// impl Actor<BlockingChatMessage, ResponseMessage> for SampleBlockingActor {
///  async fn receive(&mut self, message: Message<BlockingChatMessage, ResponseMessage>) {
///    println!("Received message: {:?}", message.payload);
///     
/// }
/// }
/// #[tokio::main]
/// async fn main() {
/// static THREAD_MONITOR: AtomicUsize = AtomicUsize::new(0);
/// static SEND_MONITOR: AtomicUsize = AtomicUsize::new(0);
/// get_loggings();
/// let mut actor_system = start_actor_system::<BlockingChatMessage, ResponseMessage>();
/// let (tx, _rx) = std::sync::mpsc::channel::<Message<BlockingChatMessage, ResponseMessage>>();
/// let actor = BlockingActorImpl::new(None, SampleBlockingActor, _rx, None);
/// let actor_ref = BlockingActorRef::new(actor, tx, &THREAD_MONITOR, &SEND_MONITOR);
/// let message = Message {payload: Some(BlockingChatMessage::Hello { name: "Alice".to_string() }), stop: false, responder: None, blocking: None};
/// actor_ref.send(message);
/// }
/// ```
///
pub struct BlockingActorRef<MType, Response>
where
    MType: Send + 'static,
    Response: Send + 'static,
{
    send_monitor: &'static AtomicUsize,
    sender: std::sync::mpsc::Sender<Message<MType, Response>>,
}

impl<MType: Send, Response: Send> BlockingActorRef<MType, Response> {
    pub fn new<T: Actor<MType, Response> + Send + 'static>(
        actor: BlockingActorImpl<T, MType, Response>,
        sender: std::sync::mpsc::Sender<Message<MType, Response>>,
        thread_monitor: &'static AtomicUsize,
        send_monitor: &'static AtomicUsize,
    ) -> Self {
        info!(actor = BLOCKING_ACTOR_NAME; "Spawning a Blocking Actor");
        std::thread::spawn(move || {
            thread_monitor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                run_a_blocking_actor::<MType, Response, T>(actor).await;
            });
        });
        info!(actor = BLOCKING_ACTOR_NAME; "Blocking Actor spawned successfully");
        BlockingActorRef {
            send_monitor,
            sender,
        }
    }
    pub fn send(&self, message: Message<MType, Response>) {
        info!(actor = BLOCKING_ACTOR_NAME; "Sending message to Blocking Actor from the BlockingActorRef.");
        self.sender
            .send(message)
            .expect("Failed to send message to blocking actor");
        self.send_monitor
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {

    use super::super::messages::Message;
    use super::*;
    use crate::actors::{
        actor::{Actor, ActorImpl, BlockingActorImpl},
        messages::ResponseMessage,
    };
    use tokio::sync::mpsc;

    use env_logger::{Builder, Env};

    fn get_loggings() {
        let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
        Builder::from_env(env).init()
    }

    struct Payload {
        message: String,
    }

    struct BlockingPayload {
        message: String,
    }

    struct SampleActor;

    impl Actor<Payload, ResponseMessage> for SampleActor {
        async fn receive(&mut self, message: Message<Payload, ResponseMessage>) {
            info!("Received the message");
            // do nothing
            info!("Received message {:?}", message.payload.unwrap().message);
            match message.responder {
                Some(responder) => {
                    responder.handle(ResponseMessage::Success).await;
                }
                None => {
                    info!("No responder found");
                }
            }
        }
    }

    struct SampleBlockingActor;

    impl Actor<BlockingPayload, ResponseMessage> for SampleBlockingActor {
        async fn receive(&mut self, message: Message<BlockingPayload, ResponseMessage>) {
            info!("Received blocking messsage");
            let Message {
                payload,
                stop: _,
                responder: _,
                blocking,
            } = message;
            info!("Received messsage : {:?}", payload.unwrap().message);
            blocking
                .expect("blocking is None.")
                .send(ResponseMessage::Success)
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_actor_ref() {
        static THREAD_MONITOR: AtomicUsize = AtomicUsize::new(0);
        static SEND_MONITOR: AtomicUsize = AtomicUsize::new(0);
        let sample = SampleActor;
        let (_tx, rx) = mpsc::channel::<Message<Payload, ResponseMessage>>(1);
        let (rs_tx, rs_rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let handler = super::super::response_handler::from_oneshot(rs_tx);
        let actor = ActorImpl::new(None, sample, rx, None);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx, &THREAD_MONITOR, &SEND_MONITOR);
        assert!(_actor_ref.sender.capacity() == 1);
        let payload = Payload {
            message: "Test".to_string(),
        };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: Some(handler),
            blocking: None,
        };
        let _ = _actor_ref.send(message).await;
        let response = rs_rx.await.expect("Failed to receive response");
        assert!(response == ResponseMessage::Success);
    }

    #[test]
    fn test_blocking_actor_ref() {
        static THREAD_MONITOR: AtomicUsize = AtomicUsize::new(0);
        static SEND_MONITOR: AtomicUsize = AtomicUsize::new(0);
        get_loggings();
        let sample = SampleBlockingActor;
        // Set up the blocking channel
        let (_tx, rx) = std::sync::mpsc::channel::<Message<BlockingPayload, ResponseMessage>>();
        let (rs_tx, rs_rx) = std::sync::mpsc::sync_channel::<ResponseMessage>(2);
        let actor = BlockingActorImpl::new(None, sample, rx, None);
        let actor_ref = BlockingActorRef::new(actor, _tx, &THREAD_MONITOR, &SEND_MONITOR);

        let payload = BlockingPayload {
            message: "Test".to_string(),
        };
        let message = Message {
            payload: Some(payload),
            stop: false,
            responder: None,
            blocking: Some(rs_tx),
        };
        actor_ref.send(message);
        assert!(rs_rx.recv().is_ok());
        let stop_message = Message {
            payload: None,
            stop: true,
            responder: None,
            blocking: None,
        };
        actor_ref.send(stop_message);
    }
}
