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
}

impl Officer {

    pub fn new(id: u32, actor: ActorRef) -> Officer {
        Officer {
            _id: id,
            actor,
            courriers: Vec::new(),
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
        for courier in self.courriers.iter_mut() {
            courier.send(message.clone()).await;
        }
        Ok(())
    }
}

pub (super) struct BlockingOfficer {
    pub (super) officer: Officer,
    pub (super) actor: BlockingActorRef,
}

impl BlockingOfficer {
    pub fn new(officer: Officer, actor: BlockingActorRef) -> BlockingOfficer {
        BlockingOfficer {
            officer,
            actor,
        }
    }

    pub fn send(&mut self, message: messages::InternalMessage) {
        info!(actor="BlockingOfficer"; "Sending message to BlockingOfficer");
        self.actor.send(message);
    }

    pub async fn notify(&mut self, message: InternalMessage) -> Result<(), std::io::Error> {
        let _ = self.officer.notify(message).await;
        Ok(())
    }

    pub fn subscribe(&mut self, actor: ActorRef) {
        self.officer.subscribe(actor);
    }

    pub fn unsubscribe(&mut self, actor_id: u32) {
        self.officer.unsubscribe(actor_id);
    }

}

// grcov-excl-start
#[cfg(test)]
mod tests {


    #[tokio::test]
    async fn test_officer() {
        
    }
    #[tokio::test]

    async fn test_blocking_officer() {
        
    }
}

// grcov-excl-stop
