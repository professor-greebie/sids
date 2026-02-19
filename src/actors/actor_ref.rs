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
    pub sender: tokio::sync::mpsc::Sender<Message<MType, Response>>,
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
/// An actor reference that requires blocking such as for retrieving data from an http request.
pub struct BlockingActorRef<MType, Response> 
where MType : Send + 'static, Response: Send + 'static {
    sender: std::sync::mpsc::Sender<Message<MType, Response>>,
}

impl <MType: Send, Response: Send> BlockingActorRef <MType, Response> {
    pub (super) fn new<T: Actor<MType, Response> + Send + 'static>(actor: BlockingActorImpl<T, MType, Response>, sender: std::sync::mpsc::Sender<Message<MType, Response>>) -> Self {
        info!(actor = "Blocking Actor"; "Spawning a Blocking Actor");
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                run_a_blocking_actor::<MType, Response, T>(actor).await;
            });
        });
        info!(actor = "Blocking Actor"; "Blocking Actor spawned successfully");
        BlockingActorRef {
            sender
        }
    }
    pub (super) fn send(&self, message: Message<MType, Response>) {
            self.sender.send(message).expect("Failed to send message to blocking actor");
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::actors::{actor::{Actor, ActorImpl, BlockingActorImpl}, messages::ResponseMessage};
    use super::super::messages::Message;
    use tokio::sync::mpsc;


    use env_logger::{Builder, Env};

    fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}


    struct Payload {
        message: String,
    }

    struct BlockingPayload {
        message: String,
    }

    struct SampleActor;
        
    impl Actor<Payload, ResponseMessage> for SampleActor {
        
        async fn receive(&mut self, message: Message<Payload, ResponseMessage>) {
            info!("Received the message");
            // do nothing
            info!("Received message {:?}", message.payload.unwrap().message);
            match message.responder {
                Some(responder) => {
                    let _ = responder.send(ResponseMessage::Success);
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
            info!("Received blocking messsage");
            let Message {payload, stop: _, responder: _, blocking} = message;
            info!("Received messsage : {:?}", payload.unwrap().message);
            blocking.expect("blocking is None.").send(ResponseMessage::Success).unwrap();
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

    #[test]
    fn test_blocking_actor_ref() { 
        get_loggings();   
        let sample = SampleBlockingActor;
        // Set up the blocking channel
        let (_tx, rx) = std::sync::mpsc::channel::<Message<BlockingPayload, ResponseMessage>>();
        let (rs_tx, rs_rx) = std::sync::mpsc::sync_channel::<ResponseMessage>(2);
        let actor = BlockingActorImpl::new(None, sample, rx);
        let actor_ref = BlockingActorRef::new( actor, _tx,);
        
        let payload = BlockingPayload { message: "Test".to_string() };
        let message = Message {payload: Some(payload), stop: false, responder: None, blocking: Some(rs_tx) };
        actor_ref.send(message);
        assert!(rs_rx.recv().is_ok());
        let stop_message = Message {payload: None, stop: true, responder: None, blocking: None };
        actor_ref.send(stop_message);
        
    }
}