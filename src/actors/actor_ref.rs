use crate::actors::actor::{run_a_blocking_actor, run_guardian};

use super::actor::{ run_an_actor, Actor, ActorImpl, BlockingActorImpl};
use super::guardian::Guardian;
use super::messages::{GuardianMessage, Message };
use log::info;



pub(super) struct GuardianActorRef {
    sender: tokio::sync::mpsc::Sender<GuardianMessage>,
}

impl GuardianActorRef {
    pub (super) fn new(actor : Guardian, sender: tokio::sync::mpsc::Sender<GuardianMessage>) -> Self {
        info!(actor = "Guardian"; "Spawning a Guardian actor");
        tokio::spawn(run_guardian(actor));
        GuardianActorRef { sender }
    }

    pub (super) async fn send(&self, message: GuardianMessage) {
        info!("Sending message to Guardian");
        self.sender.send(message).await.expect("Failed to send message to Guardian");
    }

}


#[derive(Debug)]
pub (super) struct ActorRef {
    sender: tokio::sync::mpsc::Sender<Message>,
}

impl ActorRef {
    pub fn new<T: Actor + 'static>(actor : ActorImpl<T>, sender: tokio::sync::mpsc::Sender<Message>) -> Self {
        info!(actor = "Log Actor"; "An actor is being spawned");
        tokio::spawn(run_an_actor::<T>(actor));
        ActorRef { sender }
    }

    pub async fn send(&self, message: Message) {
        let _ = self.sender.send(message).await;
    }
}

#[derive(Debug)]
pub struct BlockingActorRef {
    sender: std::sync::mpsc::Sender<Message>,
}

impl BlockingActorRef{
    pub (super) fn new<T: Actor + std::marker::Send +  'static>(actor : BlockingActorImpl<T>, sender: std::sync::mpsc::Sender<Message>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move | | {
            //
            run_a_blocking_actor(actor);
        });
        BlockingActorRef {
            sender
        }
    }
    pub fn send(&self, message: Message) {
        let _ = self.sender.send(message);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::actors::{actor::Actor, messages::ResponseMessage};
    use super::super::messages::Message;
    use tokio::sync::mpsc;

    struct SampleActor;
        
    impl Actor for SampleActor {
        fn receive(&mut self, _message: Message) {
            // do nothing
            info!("Received message");
            if let Message::StringResponse { message, responder } = _message {
                info!("Received message: {}", message);
                let _ = responder.send(ResponseMessage::Success).expect("Failed to send response");
            }
        }
    }

    struct SampleBlockingActor; 

    impl Actor for SampleBlockingActor {
        fn receive(&mut self, _message: Message) {
            // do nothing
            info!("Received message in blocking actor via ActorTrait");
            if let Message::StringResponseBlocking { message, responder } = _message {
                info!("Received message: {}", message);
                let _ = responder.send(ResponseMessage::Success);
            }
        }
    }

    #[tokio::test]
    async fn test_guardian_actor_ref() {
        // guardian channel
        let (_tx, rx) = mpsc::channel::<GuardianMessage>(1);
        let actor = Guardian::new(rx);
        let _guardian_actor_ref = GuardianActorRef::new(actor, _tx);
        assert!(_guardian_actor_ref.sender.capacity() == 1);
        let test_actor = SampleActor;
        // officer channel
        let (_tx, rx) = mpsc::channel::<Message>(1);
        let actor = ActorImpl::new(None, test_actor, rx);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx);
        // response channel
        let (rs_tx, rs_rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let message = GuardianMessage::CreateOfficer { officer_type: _actor_ref, responder: rs_tx };
        let _ = _guardian_actor_ref.send(message).await;
        let response = rs_rx.await.expect("Failed to receive response");
        assert!(response == ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_actor_ref() {
        let sample = SampleActor;
        let (_tx, rx) = mpsc::channel::<Message>(1);
        let (rs_tx, rs_rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let actor = ActorImpl::new(None, sample, rx);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx);
        assert!(_actor_ref.sender.capacity() == 1);
        let message = Message::StringResponse { message: "Test".to_string(), responder: rs_tx };
        let _ = _actor_ref.send(message).await;
        let response = rs_rx.await.expect("Failed to receive response");
        assert!(response == ResponseMessage::Success);
    }

    #[test]
    fn test_blocking_actor_ref() {
        let (_tx, rx) = std::sync::mpsc::channel::<Message>();
        let actor = BlockingActorImpl::new(None, SampleBlockingActor, rx);
        let _actor_ref = BlockingActorRef::new( actor, _tx,);
        assert!(_actor_ref.send(Message::StringMessage { message: "Test".to_string() }) == ());
        let (rs_tx, rs_rx) = std::sync::mpsc::channel::<ResponseMessage>();
        let message = Message::StringResponseBlocking { message: "Test".to_string(), responder: rs_tx };
        let _ = _actor_ref.send(message);
        let response = rs_rx.recv().expect("Failed to receive response");
        assert!(response == ResponseMessage::Success);
    }
}