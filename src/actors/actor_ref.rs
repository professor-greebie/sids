use crate::actors::actor::Actor;
use super::messages::{GetActorMessage, Message, Response};
use log::{info, error, warn};


#[derive(Clone)]
pub enum SenderType {
    TokioSender(tokio::sync::mpsc::Sender<Message>),
    StdSender(std::sync::mpsc::Sender<Message>),

}

#[derive(Clone)]

pub struct ActorRef {
    sender: SenderType,
}


impl ActorRef {

    pub fn new(actor: Actor, snd: SenderType) -> Self {
        match actor {
            Actor::Guardian(guardian) => {
                info!("Spawning guardian actor");
                tokio::spawn(async move {
                    let mut guardian = guardian;
                    guardian.run().await;
                });
            },
            Actor::GetActor(get_actor) => {
                info!("Spawning get actor with std sender");
                std::thread::spawn( move || {
                    let mut get_actor = get_actor;
                    get_actor.run();
                });
            },
            _ => {
                error!("Actor not found");
            }
        }
        Self{ sender: snd }
    }

    pub async fn send(&mut self, message: Message) {
        
        match &self.sender {
            SenderType::TokioSender(sender) =>  {
                let _ = sender.send(message).await;
            },
            SenderType::StdSender(_sender) => {
                warn!("Std sender should not be sent via async implementation");
                
            }
        };
    }

    pub fn get_uri_result(self, uri: String, location: String) -> () {
        let (snd, rec) = std::sync::mpsc::channel::<Message>();
        match self.sender {
            SenderType::TokioSender(_sender) => {
                warn!("Tokio sender should not be sent via sync implementation")

            },
            SenderType::StdSender(sender) => {
                let _ = sender.send(Message::GetActorMessage(GetActorMessage::GetURI { uri, location, responder: snd })).unwrap();
            }
        }
        let response = rec.recv().unwrap();
        match response {
            Message::Response(Response::Success) => {
                info!("Success");
            },
            Message::Response(Response::Failure) => {
                error!("Failure");
            },
            _ => {
                error!("No response received");
            }
        }
    }
}



