use std::sync::atomic::AtomicUsize;

use crate::actors::actor::run_a_blocking_actor;
use super::actor::{ run_an_actor, Actor, ActorImpl, BlockingActorImpl};
use super::messages::Message;
use log::info;



#[derive(Debug)]
pub struct ActorRef<MType> 
where MType: Send + 'static {
    send_monitor: &'static AtomicUsize,
    sender: tokio::sync::mpsc::Sender<Message<MType>>,
}

impl <MType: Send> ActorRef <MType> {
    pub fn new<T: Actor<MType> + 'static>(actor : ActorImpl<T, MType>, sender: tokio::sync::mpsc::Sender<Message<MType>>, thread_monitor: &'static AtomicUsize, send_monitor: &'static AtomicUsize) -> Self {
        info!(actor = "Log Actor"; "An actor is being spawned");
        tokio::spawn(async move {
            thread_monitor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            run_an_actor::<MType, T>(actor).await;
            
        });
        ActorRef { send_monitor, sender }
    }

    pub async fn send(&self, message: Message<MType>) {

        let _ = self.sender.send(message).await;
        self.send_monitor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn sender(&self) -> tokio::sync::mpsc::Sender<Message<MType>> {
        self.sender.clone()
    }
}

#[derive(Debug)]
pub struct BlockingActorRef<R> 
where R : Send + 'static {
    sender: std::sync::mpsc::Sender<Message<R>>,
}

impl <R: Send> BlockingActorRef <R> {
    pub (super) fn new<T: Actor<R> + std::marker::Send +  'static>(actor : BlockingActorImpl<T, R>, sender: std::sync::mpsc::Sender<Message<R>>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move  | | async {
            //
            run_a_blocking_actor(actor).await;
        });
        BlockingActorRef {
            sender
        }
    }
    pub (super) fn send(&self, message: Message<R>) {
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
        
    impl Actor<Payload> for SampleActor {
        
        async fn receive(&mut self, message: Message<Payload>) {
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

    impl Actor<BlockingPayload> for SampleBlockingActor {
        async fn receive(&mut self, message: Message<BlockingPayload>) {
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
        let (_tx, rx) = mpsc::channel::<Message<Payload>>(1);
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
        let (_tx, rx) = std::sync::mpsc::channel::<Message<BlockingPayload>>();
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