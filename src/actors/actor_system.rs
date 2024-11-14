use std::collections::HashMap;

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

impl<MType> Actor<MType> for Guardian {
    fn receive(&mut self, message: Message<MType>) {
        info!("Guardian received a message");
        if message.stop {
            info!("Guardian received a stop message");
        }
        match message.responder {
            Some(responder) => {
                let _ = responder
                    .send(ResponseMessage::SUCCESS)
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
/// use sids::actors::actor_system::ActorSystem;
/// use sids::actors::messages::{Message, ResponseMessage};
///
/// pub async fn run_system() {
///
///     let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
/// }
///
/// ```
pub struct ActorSystem<MType: Send + 'static> {
    actors: HashMap<u32, ActorRef<MType>>,
    blocking_actors: HashMap<u32, BlockingActorRef<MType>>,
    snd: mpsc::Sender<Message<MType>>,
}

impl<MType: Send + 'static> ChannelFactory<MType> for ActorSystem<MType> {
    fn create_actor_channel(
        &self,
    ) -> (
        tokio::sync::mpsc::Sender<Message<MType>>,
        tokio::sync::mpsc::Receiver<Message<MType>>,
    ) {
        mpsc::channel::<Message<MType>>(super::SIDS_DEFAULT_BUFFER_SIZE)
    }

    fn create_blocking_actor_channel(
        &self,
    ) -> (
        std::sync::mpsc::Sender<Message<MType>>,
        std::sync::mpsc::Receiver<Message<MType>>,
    ) {
        std::sync::mpsc::channel::<Message<MType>>()
    }

    fn create_response_channel(
        &self,
    ) -> (
        tokio::sync::oneshot::Sender<ResponseMessage>,
        tokio::sync::oneshot::Receiver<ResponseMessage>,
    ) {
        oneshot::channel::<ResponseMessage>()
    }

    fn create_blocking_response_channel(
        &self,
    ) -> (
        std::sync::mpsc::Sender<ResponseMessage>,
        std::sync::mpsc::Receiver<ResponseMessage>,
    ) {
        std::sync::mpsc::channel::<ResponseMessage>()
    }
}

impl<MType: Send + 'static> ActorSystem<MType> {
    /// Create a new ActorSystem
    ///
    /// The ActorSystem will start by launching a guardian, which is a non-blocking officer-actor that manages all other actors in the system.
    /// The guardian will be dormant until start_system is called in the ActorSystem.
    pub(super) fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Message<MType>>(super::SIDS_DEFAULT_BUFFER_SIZE);
        info!(actor = "guardian"; "Guardian channel and actor created. Launching...");
        info!(actor = "guardian"; "Guardian actor spawned");
        info!(actor = "guardian"; "Actor system created");
        let guardian = ActorImpl::new(Some("Guardian Type".to_string()), Guardian, rx);
        let actor_ref = ActorRef::new(guardian, tx.clone());
        let mut actors = HashMap::new();
        let blocking_actors = HashMap::new();
        actors.insert(0, actor_ref);
        ActorSystem {
            actors: actors,
            blocking_actors: blocking_actors,
            snd: tx,
        }
    }

    pub(super) async fn spawn_actor<T>(&mut self, actor: T, name: Option<String>)
    where
        T: Actor<MType> + 'static,
    {
        info!("Spawning actor");
        let (snd, rec) = self.create_actor_channel();
        let actor_impl = ActorImpl::new(name, actor, rec);
        let actor_ref = ActorRef::new(actor_impl, snd);
        let actor_id = self.actors.len() as u32;
        self.actors.insert(actor_id, actor_ref);
    }

    pub (super) fn spawn_blocking_actor<T>(&mut self, actor: T, name: Option<String>)
    where
        T: Actor<MType> + 'static,
    {
        info!("Spawning blocking actor");
        let (snd, rec) = self.create_blocking_actor_channel();
        let actor_impl = BlockingActorImpl::new(name, actor, rec);
        let actor_ref = BlockingActorRef::new(actor_impl, snd);
        let actor_id = self.blocking_actors.len() as u32;
        self.blocking_actors.insert(actor_id, actor_ref);
    }

    pub(super) async fn send_message_to_actor(&mut self, actor_id: u32, message: Message<MType>) {
        if let Message { payload: _, stop: _, responder: None, blocking: Some(_) } = &message {
            let blocking_actor = self.blocking_actors.get_mut(&actor_id).expect("Failed to get blocking actor");
                blocking_actor.send(message);
        } else if let Message { payload: _, stop: _, responder: _, blocking: None } = &message {
            let actor = self.actors.get_mut(&actor_id).expect("Failed to get actor");
            actor.send(message).await;
        } else {
            warn!("No actor found with id: {}", actor_id);
        }
        
    }

    pub(super) async fn ping_system(&self) {
        info!("Pinging system");
        self.snd.send(Message{payload: None, stop: false, responder: None, blocking: None}).await.expect("Failed to send message");
    }
}

// grcov-excl-start

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_actor_system_started() {
    }
}

// grcov-excl-stop
