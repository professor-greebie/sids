use crate::actors::actor::{run_a_blocking_actor, run_guardian};

use super::actor::{ run_an_actor, Actor, ActorTrait, BlockingActor};
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
        info!("Sending message to Guardian");
        self.sender.send(message).await.expect("Failed to send message to Guardian");
    }

}


#[derive(Debug)]
pub (super) struct ActorRef {
    sender: tokio::sync::mpsc::Sender<InternalMessage>,
}

impl ActorRef {
    pub fn new<T: ActorTrait + 'static>(actor : Actor<T>, sender: tokio::sync::mpsc::Sender<InternalMessage>) -> Self {
        info!(actor = "Log Actor"; "An actor is being spawned");
        tokio::spawn(run_an_actor::<T>(actor));
        ActorRef { sender }
    }

    pub async fn send(&self, message: InternalMessage) {
        let _ = self.sender.send(message).await;
    }
}

#[derive(Debug)]
pub struct BlockingActorRef {
    sender: std::sync::mpsc::Sender<InternalMessage>,
}

impl BlockingActorRef{
    pub (super) fn new<T: ActorTrait + std::marker::Send +  'static>(actor : BlockingActor<T>, sender: std::sync::mpsc::Sender<InternalMessage>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move | | {
            //
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

    use super::*;
    use crate::actors::actor::ActorTrait;
    use super::super::messages::InternalMessage;
    use tokio::sync::mpsc;

    struct SampleActor;
        
    impl ActorTrait for SampleActor {
        fn receive(&mut self, _message: InternalMessage) {
            // do nothing
            info!("Received message");
        }
    }

    struct SampleBlockingActor; 

    impl ActorTrait for SampleBlockingActor {
        fn receive(&mut self, _message:InternalMessage) {
            // do nothing
            info!("Received message in blocking actor via ActorTrait");
        }
    }

    #[tokio::test]
    async fn test_actor_ref() {
        let sample = SampleActor;
        let (_tx, rx) = mpsc::channel::<InternalMessage>(1);
        let actor = Actor::new(None, sample, rx);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx);
        assert!(_actor_ref.sender.capacity() == 1);
    }

    #[test]
    fn test_blocking_actor_ref() {
        let (_tx, rx) = std::sync::mpsc::channel::<InternalMessage>();
        let actor = BlockingActor::new(None, SampleBlockingActor, rx);
        let _actor_ref = BlockingActorRef::new( actor, _tx,);
        assert!(_actor_ref.send(InternalMessage::StringMessage { message: "Test".to_string() }) == ());
        //assert!(_actor_ref.sender == InternalMessage::LogMessage { message: "Test".to_string() }.type_id());
    }
}