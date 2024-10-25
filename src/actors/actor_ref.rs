use crate::actors::actor::run_a_blocking_actor;

use super::actor::{ run_an_actor, Actor, ActorImpl, BlockingActorImpl};
use super::messages::{ActorType, Message};
use log::info;


#[derive(Debug)]
pub (super) struct ActorRef<R> 
where R: Send + 'static {
    sender: tokio::sync::mpsc::Sender<Message<ActorType, R>>,
}

impl <R: Send> ActorRef <R> {
    pub fn new<T: Actor<R> + 'static>(actor : ActorImpl<T, R>, sender: tokio::sync::mpsc::Sender<Message<ActorType, R>>) -> Self {
        info!(actor = "Log Actor"; "An actor is being spawned");
        tokio::spawn(run_an_actor::<R, T>(actor));
        ActorRef { sender }
    }

    pub async fn send(&self, message: Message<ActorType, R>) {
        let _ = self.sender.send(message).await;
    }
}

#[derive(Debug)]
pub struct BlockingActorRef<R> 
where R : Send + 'static {
    sender: std::sync::mpsc::Sender<Message<ActorType, R>>,
}

impl <R: Send> BlockingActorRef <R> {
    pub (super) fn new<T: Actor<R> + std::marker::Send +  'static>(actor : BlockingActorImpl<T, R>, sender: std::sync::mpsc::Sender<Message<ActorType, R>>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move | | {
            //
            run_a_blocking_actor(actor);
        });
        BlockingActorRef {
            sender
        }
    }
    pub fn send(&self, message: Message<ActorType, R>) {
        let _ = self.sender.send(message);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::actors::{actor::{Actor, ActorImpl, BlockingActorImpl}, messages::{AnyActor, ResponseMessage}};
    use super::super::messages::Message;
    use tokio::sync::mpsc;

    struct Payload {
        message: String,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    }

    struct BlockingPayload {
        message: String,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    }

    struct SampleActor;
        
    impl Actor<Payload> for SampleActor {
        
        fn receive(&mut self, message: Message<ActorType, Payload>) {
            // do nothing
            info!("Received message");
            let _ = message.payload.responder.send(ResponseMessage::SUCCESS).expect("Failed to send response");
        }
    }

    struct SampleBlockingActor; 

    impl Actor<BlockingPayload> for SampleBlockingActor {
        fn receive(&mut self, message: Message<ActorType, BlockingPayload>) {
            // do nothing
            let _ = message.payload.responder.send(ResponseMessage::SUCCESS).expect("Failed to send response");
        }
    }

    #[tokio::test]
    async fn test_actor_ref() {
        let sample = SampleActor;
        let (_tx, rx) = mpsc::channel::<Message<ActorType, Payload>>(1);
        let (rs_tx, rs_rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let actor = ActorImpl::new(None, sample, rx);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx);
        assert!(_actor_ref.sender.capacity() == 1);
        let payload = Payload { message: "Test".to_string(), responder: rs_tx };
        let message = Message { for_actor: ActorType::Officer(AnyActor), payload: payload };
        let _ = _actor_ref.send(message).await;
        let response = rs_rx.await.expect("Failed to receive response");
        assert!(response == ResponseMessage::SUCCESS);
    }

    #[test]
    fn test_blocking_actor_ref() {
        let sample = SampleBlockingActor;
        let (_tx, rx) = std::sync::mpsc::channel::<Message<ActorType, BlockingPayload>>();
        let (rs_tx, rs_rx) = std::sync::mpsc::channel::<ResponseMessage>();
        let actor = BlockingActorImpl::new(None, sample, rx);
        let actor_ref = BlockingActorRef::new( actor, _tx,);
        
        let payload = BlockingPayload { message: "Test".to_string(), responder: rs_tx };
        let message = Message { for_actor: ActorType::Officer(AnyActor), payload: payload };
        let _ = actor_ref.send(message);
        let response = rs_rx.recv().expect("Failed to receive response");
        assert!(response == ResponseMessage::SUCCESS);
    }
}