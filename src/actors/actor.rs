use super::messages::Message;
use log::info;
use tokio::sync::mpsc;


/// The main actor trait that all actors must implement.
#[trait_variant::make(Send)]
pub trait Actor<MType, Response>{
    async fn receive(&mut self, message: Message<MType, Response>) where Self: Sized + 'static;
}


/// Helper function for running an actor.
pub (super) async fn run_an_actor<MType, Response, T: Actor<MType, Response> + 'static>(mut actor : ActorImpl <T, MType, Response>) {
    while let Some(message) = actor.receiver.recv().await {
        if message.stop {
            break;
        }
        actor.receive(message).await;
    }

}

/// Helper function for running a blocking actor.
pub (super) async fn run_a_blocking_actor<MType, Response, T: Actor<MType, Response> + 'static>(mut actor: BlockingActorImpl<T, MType, Response>) {
    while let Ok(message) = actor.receiver.recv() {
        if message.stop {
            break;
        }
        actor.receive(message).await;
    }
}


/// Implements an actor with an Actor type T, and a Payload type R (can be the NotUsed type if no payload is received).
pub struct ActorImpl<T, MType, Response> {
    name: Option<String>,
    actor: T,
    receiver: mpsc::Receiver<Message<MType, Response>>,
}

impl <MType, Response, T: Actor<MType, Response> + 'static> ActorImpl <T, MType, Response> {
    pub async fn receive(&mut self, message: Message<MType, Response>){
        info!("Actor {} received message", self.name.clone().unwrap_or("Unnamed Actor".to_string()));
        T::receive(&mut self.actor, message).await;
    }
    pub fn new (name: Option<String>, actor: T, receiver: mpsc::Receiver::<Message<MType, Response>>) -> Self {

        ActorImpl {name, actor, receiver }
    }
}

pub struct BlockingActorImpl<T, MType, Response> {
    name: Option<String>,
    actor: T,
    receiver: std::sync::mpsc::Receiver<Message<MType, Response>>,
}

impl <MType, Response, T: Actor<MType, Response> + 'static> BlockingActorImpl<T, MType, Response> {

     pub async fn receive(&mut self, message: Message<MType, Response>){ 
        info!("Blocking actor {} received message", self.name.clone().unwrap_or("Unnamed Blocking Actor".to_string()));
        T::receive(&mut self.actor, message).await;
    }

    pub fn new (name: Option<String>, actor: T, receiver: std::sync::mpsc::Receiver<Message<MType, Response>>) -> BlockingActorImpl<T, MType, Response> {
        BlockingActorImpl { name, actor, receiver}
    }
}