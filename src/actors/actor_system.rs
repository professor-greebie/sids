use std::{collections::HashMap, sync::atomic::AtomicUsize};

use crate::actors::actor::{ActorImpl, BlockingActorImpl};

use super::{
    actor::Actor,
    actor_ref::{ActorRef, BlockingActorRef},
    channel_factory::ChannelFactory,
    messages::{Message, ResponseMessage},
};
use log::{info, warn};
use tokio::sync::{mpsc, oneshot};

struct Guardian;

impl<MType: Send + Clone> Actor<MType, ResponseMessage> for Guardian {
    async fn receive(&mut self, message: Message<MType, ResponseMessage>) {
        info!("Guardian received a message");
        if message.stop {
            info!("Guardian received a stop message");
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
}

impl<MType: Send + Clone + 'static, Response: Send + Clone + 'static> ChannelFactory<MType, Response> for ActorSystem<MType, Response> {
    fn create_actor_channel(
        &self,
    ) -> (
        tokio::sync::mpsc::Sender<Message<MType, Response>>,
        tokio::sync::mpsc::Receiver<Message<MType, Response>>,
    ) {
        mpsc::channel::<Message<MType, Response>>(super::SIDS_DEFAULT_BUFFER_SIZE)
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
        let (tx, rx) = mpsc::channel::<Message<MType, ResponseMessage>>(super::SIDS_DEFAULT_BUFFER_SIZE);
        info!(actor = "guardian"; "Guardian channel and actor created. Launching...");
        info!(actor = "guardian"; "Guardian actor spawned");
        let guardian = ActorImpl::new(Some("Guardian Type".to_string()), Guardian, rx);
        info!(actor = "guardian"; "Actor system created");
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
        }
    }

    pub async fn spawn_actor<T>(&mut self, actor: T, name: Option<String>)
    where
        T: Actor<MType, Response> + 'static,
    {
        info!("Spawning actor within the actor system.");
        let (snd, rec) = self.create_actor_channel();
        let actor_impl = ActorImpl::new(name, actor, rec);
        let actor_ref = ActorRef::new(actor_impl, snd, self.total_threads, self.total_messages);
        let actor_id = self.actors.len() as u32;
        self.actors.insert(actor_id, actor_ref);
        info!("Actor spawned successfully with id: {}", actor_id);
    }

    pub(super) fn spawn_blocking_actor<T>(&mut self, actor: T, name: Option<String>)
    where
        T: Actor<MType, Response> + 'static,
    {
        info!("Spawning blocking actor within the actor system.");
        let (snd, rec) = self.create_blocking_actor_channel();
        let actor_impl = BlockingActorImpl::new(name, actor, rec);
        let actor_ref = BlockingActorRef::new(actor_impl, snd);
        let actor_id = self.blocking_actors.len() as u32;
        self.blocking_actors.insert(actor_id, actor_ref);
        info!("Blocking actor spawned successfully with id: {}", actor_id);
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
        } else if let Message {
            payload: _,
            stop: _,
            responder: _,
            blocking: None,
        } = &message
        {
            let actor = self.actors.get_mut(&actor_id).expect("Failed to get actor");
            actor.send(message).await;
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
}

