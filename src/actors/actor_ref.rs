use crate::actors::actor::run_guardian;

use super::actor::{run_a_blocking_actor, run_an_actor, Actor, ActorTrait, BlockingActor, BlockingActorTrait};
use super::guardian::Guardian;
use super::messages::{InternalMessage, Message };
use log::info;



pub(super) struct GuardianActorRef {
    sender: tokio::sync::mpsc::Sender<Message>,
}

impl GuardianActorRef {
    pub fn new(actor : Guardian, sender: tokio::sync::mpsc::Sender<Message>) -> Self {
        info!(actor = "Guardian"; "Spawning a Guardian actor");
        tokio::spawn(run_guardian(actor));
        GuardianActorRef { sender }
    }

    pub async fn send(&self, message: Message) {
        let _ = self.sender.send(message).await;
        // send message to guardian actor.

        

    }

}


pub (super) struct ActorRef {
    sender: tokio::sync::mpsc::Sender<InternalMessage>,
}

impl ActorRef {
    pub fn new<T: ActorTrait + 'static>(actor : Actor<T>, sender: tokio::sync::mpsc::Sender<InternalMessage>) -> Self {
        info!(actor = "Log Actor"; "Spawning a Log actor");
        tokio::spawn(run_an_actor::<T>(actor));
        ActorRef { sender }
    }

    pub async fn send(&self, message: InternalMessage) {
        let _ = self.sender.send(message).await;
    }
}

pub struct BlockingActorRef {
    sender: std::sync::mpsc::Sender<InternalMessage>,
}

impl BlockingActorRef{
    pub fn new<T: BlockingActorTrait + std::marker::Send +  'static>(actor : BlockingActor<T>, sender: std::sync::mpsc::Sender<InternalMessage>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move | | {
            run_a_blocking_actor(actor);
        });
        BlockingActorRef {
            sender
        }
    }
    pub fn send(&self, message: InternalMessage) {
        let _ = self.sender.send(message);
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;
    use crate::actors::actor::{blocking_actor_receive, ActorTrait, BlockingActorTrait};
    use super::super::messages::InternalMessage;
    use tokio::sync::mpsc;

    struct SampleActor;
        
    impl ActorTrait for SampleActor {
        async fn receive(&mut self, _message: InternalMessage) {
            // do nothing
            info!("Received message");
        }
    }

    struct SampleBlockingActor; 

    impl ActorTrait for SampleBlockingActor {
        async fn receive(&mut self, message:InternalMessage) {
            blocking_actor_receive(self, message).await;
        }
    }

    impl BlockingActorTrait for SampleBlockingActor  {
        
        fn handle_message(&mut self, message: InternalMessage) {
            info!("Received message");
        }
    }

    #[tokio::test]
    async fn test_actor_ref() {
        let sample = SampleActor;
        let (_tx, rx) = mpsc::channel::<InternalMessage>(1);
        let actor = Actor::new(sample, rx);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx);
        assert!(_actor_ref.sender.capacity() == 1);
    }

    #[test]
    fn test_blocking_actor_ref() {
        let (_tx, rx) = std::sync::mpsc::channel::<InternalMessage>();
        let actor = BlockingActor::new(SampleBlockingActor, rx);
        let _actor_ref = BlockingActorRef::new(actor, _tx);
        //assert!(_actor_ref.sender.type_id() == InternalMessage::LogMessage { message: "Test".to_string() }.type_id());
    }
}