use crate::actors::messages::ActorMessage;

use tokio::sync::{oneshot, mpsc};
use crate::actors::actor::{Actor, Guardian};
use crate::actors::messages::GuardianMessage;

pub trait ActorRef {
    type T: Actor + 'static;
    type M: ActorMessage;
    fn new() -> Self;
    async fn tell(&self) -> u64;
    // possibly include an actor path reference to see where the message is coming from
}

#[derive(Clone)]
struct GuardianActorRef {
    sender: tokio::sync::mpsc::Sender<GuardianMessage>,
}

async fn run_the_guardian(mut actor: Guardian) {
    while let Some(message) = actor.receiver.recv().await {
        actor.receive(message);
    }
}

impl ActorRef for GuardianActorRef {
    type T = Guardian;
    type M = GuardianMessage;
    fn new() -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(1);
        let actor = Guardian::new(rec);
        tokio::spawn(run_the_guardian(actor));
        
        Self { sender: snd }
    }
    async fn tell(&self) -> u64 {
        let (sender, receiver) = oneshot::channel();
        let msg = GuardianMessage::GetNextId { responder: sender, };
        
        let _ = self.sender.send(msg).await;
        receiver.await.expect("Actor is dead")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_ref() {
        let guardian = GuardianActorRef::new();
        let test = guardian.tell();
    }
}
