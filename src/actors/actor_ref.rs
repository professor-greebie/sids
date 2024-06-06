use crate::actors::messages::ActorMessage;

use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use crate::actors::actor::{Actor, Guardian};
use crate::actors::messages::GuardianMessage;

pub trait ActorRef {
    type T: Actor + 'static;
    type M: ActorMessage;
    fn new(&self, actor: Box<Self::T>) -> Self;
    fn tell(&self, msg: Self::M) -> Result<(), std::sync::mpsc::SendError<Self::M>>;
    // possibly include an actor path reference to see where the message is coming from
}

struct GuardianActorRef {
    actor: Box<Guardian>,
}


impl ActorRef for GuardianActorRef {
    type T = Guardian;
    type M = GuardianMessage;
    fn new(&self, actor: Box<Guardian>) -> GuardianActorRef {
        GuardianActorRef {
            actor: actor,
        }
    }
    fn tell(&self, msg: GuardianMessage) -> Result<(), std::sync::mpsc::SendError<GuardianMessage>> {
        let actor = Arc::new(self.actor.clone());
        let (snd, rec) = sync_channel(1);
        let _send = std::thread::spawn(move ||  {
            snd.send(msg).unwrap();
        });
        let _receive = actor.receive(rec);
        _send.join().unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_ref() {
        let guardian = Box::new(Guardian{});
        let actor_ref = GuardianActorRef{actor: guardian};
        let test = actor_ref.tell(GuardianMessage::Terminated);
        let expected = Ok(());
        assert_eq!(test, expected);
    }
}
