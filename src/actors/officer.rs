use log::info;

use super::actor::Actor;
use super::actor_ref::{ActorRef, BlockingActorRef};
use super::messages::{ActorType, Message};



trait ChainLink {
    fn next<T: Send>(&mut self, next: ActorRef<T>);
    fn handle<R>(&self, message: Message<ActorType, R>);
}

/// The Officer struct is an actor that controls a number of courriers in an actor system.
///
/// The intention of the officer is multi-faceted. It can be used to control the state of the courriers,
/// and also to send messages to the courriers. The officer is also an actor, and can be used to send messages to itself.
/// Officers will be controlled by the guardian actor, which is the main actor in the system.
pub(super) struct Officer<T: Send + 'static> {
    pub(super) _id: u32,
    pub(super) actor: ActorRef<T>,
    pub(super) courriers: Vec<ActorRef<T>>,
}

impl <T: Send + 'static> Officer<T> {

    pub fn new(id: u32, actor: ActorRef<T>) -> Officer<T> {
        Officer {
            _id: id,
            actor,
            courriers: Vec::new(),
        }
    }

    pub async fn send(&mut self, message: Message<ActorType, T>) {
        info!(actor="officer"; "Sending message to officer {}.", self._id);
        self.actor.send(message).await;
    }

    /// Add a courrier to the officer's list of courriers.
    pub fn subscribe(&mut self, actor: ActorRef<T>) {
        self.courriers.push(actor);
    }

    /// Remove a courrier from the officer's list of courriers.
    pub fn unsubscribe(&mut self, actor_id: u32) {
        self.courriers.remove(actor_id as usize);
    }


    /// Send a message to all courriers.
    pub fn notify<R>(&mut self, _message: Message<ActorType, R>) -> Result<(), std::io::Error> {
        for _courier in self.courriers.iter_mut() {
            // Need to provide some abstraction for courriers to receive an update broadcast.
        }
        Ok(())
    }
}

pub (super) struct BlockingOfficer<T: Send + 'static> {
    pub (super) officer: Officer<T>,
    pub (super) actor: BlockingActorRef<T>,
}

impl <T: Send + 'static> BlockingOfficer<T> {
    pub fn new(officer: Officer<T>, actor: BlockingActorRef<T>) -> BlockingOfficer<T> {
        BlockingOfficer {
            officer,
            actor,
        }
    }

    pub fn send(&mut self, message: Message<ActorType, T>) {
        info!(actor="BlockingOfficer"; "Sending message to BlockingOfficer");
        self.actor.send(message);
    }

    pub fn notify(&mut self, message: Message<ActorType, T>) -> Result<(), std::io::Error> {
        let _ = self.officer.notify(message);
        Ok(())
    }

    pub fn subscribe(&mut self, actor: ActorRef<T>) {
        self.officer.subscribe(actor);
    }

    pub fn unsubscribe(&mut self, actor_id: u32) {
        self.officer.unsubscribe(actor_id);
    }

}

struct Courrier<T: Send + 'static> {
    id: u32,
    actor: ActorRef<T>,
}

impl <T: Send> ChainLink for Courrier<T> {
    fn next<U: Send>(&mut self, next: ActorRef<U>) {
        // do nothing
    }

    fn handle<U>(&self, message: Message<ActorType, U>) {
        // do nothing
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
