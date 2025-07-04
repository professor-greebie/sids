use std::sync::atomic::AtomicUsize;

use crate::actors::actor::run_a_blocking_actor;
use super::actor::{ run_an_actor, Actor, ActorImpl, BlockingActorImpl};
use super::messages::Message;
use log::info;



#[derive(Debug, Clone)]
pub struct ActorRef<MType, Response> 
where MType: Send + 'static, 
Response: Send + 'static {
    send_monitor: &'static AtomicUsize,
    sender: tokio::sync::mpsc::Sender<Message<MType, Response>>,
}

impl <MType: Send, Response: Send> ActorRef <MType, Response> {
    pub fn new<T: Actor<MType, Response> + 'static>(actor : ActorImpl<T, MType, Response>, sender: tokio::sync::mpsc::Sender<Message<MType, Response>>, thread_monitor: &'static AtomicUsize, send_monitor: &'static AtomicUsize) -> Self {
        info!(actor = "Tokio Actor"; "Spawning a Tokio Actor");
        tokio::spawn(async move {
            thread_monitor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            run_an_actor::<MType, Response, T>(actor).await;          
        });
        info!(actor = "Tokio Actor"; "Actor spawned successfully");
        ActorRef { send_monitor, sender }
    }

    pub async fn send(&self, message: Message<MType, Response>) {
        let _ = self.sender.send(message).await;
        self.send_monitor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

}

#[derive(Debug)]
pub struct BlockingActorRef<MType, Response> 
where MType : Send + 'static, Response: Send + 'static {
    sender: std::sync::mpsc::Sender<Message<MType, Response>>,
}

impl <MType: Send, Response: Send> BlockingActorRef <MType, Response> {
    pub (super) fn new<T: Actor<MType, Response> + Send + 'static>(actor: BlockingActorImpl<T, MType, Response>, sender: std::sync::mpsc::Sender<Message<MType, Response>>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move || async {
            run_a_blocking_actor(actor).await;
        });
        info!(actor = "Blocking Actor"; "Blocking Actor spawned successfully");
        BlockingActorRef {
            sender
        }
    }
    pub (super) fn send(&self, message: Message<MType, Response>) {
        let _ = self.sender.send(message);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::actors::{actor::{Actor, ActorImpl, BlockingActorImpl}, messages::ResponseMessage};
    use super::super::messages::Message;
    use tokio::sync::mpsc;

    struct Payload {
        message: String,
    }

    struct BlockingPayload {
        message: String,
    }

    struct SampleActor;
        
    impl Actor<Payload, ResponseMessage> for SampleActor {
        
        async fn receive(&mut self, message: Message<Payload, ResponseMessage>) {
            // do nothing
            info!("Received message {:?}", message.payload.unwrap().message);
            match message.responder {
                Some(responder) => {
                    let _ = responder.send(ResponseMessage::Success).expect("Failed to send response");
                }
                None => {
                    info!("No responder found");
                }
            }
        }
    }

    struct SampleBlockingActor; 

    impl Actor<BlockingPayload, ResponseMessage> for SampleBlockingActor {
        async fn receive(&mut self, message: Message<BlockingPayload, ResponseMessage>) {
            info!("Received message {:?}", message.payload.unwrap().message);
            match message.blocking {
                Some(blocking) => {
                    let _ = blocking.send(ResponseMessage::Success).expect("Failed to send response");
                }
                None => {
                    info!("No responder found");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_actor_ref() {
        static THREAD_MONITOR: AtomicUsize = AtomicUsize::new(0);
        static SEND_MONITOR: AtomicUsize = AtomicUsize::new(0);
        let sample = SampleActor;
        let (_tx, rx) = mpsc::channel::<Message<Payload, ResponseMessage>>(1);
        let (rs_tx, rs_rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let actor = ActorImpl::new(None, sample, rx);
        let _actor_ref = ActorRef::new::<SampleActor>(actor, _tx, &THREAD_MONITOR, &SEND_MONITOR);
        assert!(_actor_ref.sender.capacity() == 1);
        let payload = Payload { message: "Test".to_string()};
        let message = Message {payload: Some(payload), stop: false, responder: Some(rs_tx), blocking: None};
        let _ = _actor_ref.send(message).await;
        let response = rs_rx.await.expect("Failed to receive response");
        assert!(response == ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_blocking_actor_ref() {
        
        
        let sample = SampleBlockingActor;
        let (_tx, rx) = std::sync::mpsc::channel::<Message<BlockingPayload, ResponseMessage>>();
        let (rs_tx, rs_rx) = std::sync::mpsc::channel::<ResponseMessage>();
        let actor = BlockingActorImpl::new(None, sample, rx);
        let actor_ref = BlockingActorRef::new( actor, _tx,);
        
        let payload = BlockingPayload { message: "Test".to_string() };
        let message = Message {payload: Some(payload), stop: false, responder: None, blocking: Some(rs_tx) };
        let _ = actor_ref.send(message);
        let response = rs_rx.recv().expect("Failed to receive response");
        assert!(response == ResponseMessage::Success);
    }
}