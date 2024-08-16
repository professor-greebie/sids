use log::info;

use super::actor_ref::{ActorRef, BlockingActorRef};
use super::messages::InternalMessage;
use super::messages;




/// The Officer struct is an actor that controls a number of courriers in an actor system.
///
/// The intention of the officer is multi-faceted. It can be used to control the state of the courriers,
/// and also to send messages to the courriers. The officer is also an actor, and can be used to send messages to itself.
/// Officers will be controlled by the guardian actor, which is the main actor in the system.
pub(super) struct Officer {
    pub(super) _id: u32,
    pub(super) actor: ActorRef,
    pub(super) courriers: Vec<ActorRef>,
    pub(super) blocking_courriers: Vec<BlockingActorRef>,
}

impl Officer {

    pub fn new(actor: ActorRef) -> Officer {
        Officer {
            _id: 0,
            actor,
            courriers: Vec::new(),
            blocking_courriers: Vec::new(),
        }
    }

    pub async fn send(&mut self, message: messages::InternalMessage) {
        info!(actor="officer"; "Sending message to officer {}.", self._id);
        self.actor.send(message).await;
    }

    /// Add a courrier to the officer's list of courriers.
    pub fn subscribe(&mut self, actor: ActorRef) {
        self.courriers.push(actor);
    }
    /// Remove a courrier from the officer's list of courriers.
    pub fn unsubscribe(&mut self, actor_id: u32) {
        self.courriers.remove(actor_id as usize);
    }
    /// Send a message to all courriers.
    pub async fn notify(&mut self, message: InternalMessage) -> Result<(), std::io::Error> {
        let message_copy = message;
        self.notify_blocking_courriers(message_copy.clone());
        self.notify_tokio_courriers(message_copy).await;
        Ok(())
    }
    /// Send a message to all blocking courriers.
    pub fn notify_blocking_courriers(&mut self, message: InternalMessage) {
        for courier in self.blocking_courriers.iter_mut() {
            courier.send(message.clone());
        }
    }
    /// Send a message to all tokio (async)courriers.
    pub async fn notify_tokio_courriers(&mut self, message: messages::InternalMessage) {
        for courier in self.courriers.iter_mut() {
            courier.send(message.clone()).await;
        }
    }
}

// grcov-excl-start
#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_officer() {
        
    }
    #[tokio::test]

    async fn test_blocking_officer() {
        
    }
}

// grcov-excl-stop
