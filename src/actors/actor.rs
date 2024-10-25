use super::{actor_ref::ActorRef, guardian::Guardian, messages::{ActorType, Message, NotUsed}};
use super::community::generic::Dummy;
use log::info;
use tokio::sync::mpsc;


pub (super) fn create_dummy_actor() -> ActorRef<NotUsed> {
    let (tx, rx) = mpsc::channel(1);
    let actor = ActorImpl::new(None, Dummy, rx);
    ActorRef::new(actor, tx)
}


/// The main actor trait that all actors must implement.
#[trait_variant::make(Send)]
pub trait Actor<R>{
    fn receive(&mut self, message: Message<ActorType, R>) where Self: Sized;
}

/// Helper function for running the guardian actor.
/// 
/// This starts the overall system.
pub (super) async fn run_guardian(mut actor: Guardian) {
    while let Some(message) = actor.receiver.recv().await {
        actor.receive(message).await;
    }
}

/// Helper function for running an actor.
pub (super) async fn run_an_actor<R, T: Actor<R> + 'static>(mut actor : ActorImpl <T, R>) {
    while let Some(message) = actor.receiver.recv().await {
        info!("Running an actor");
        actor.receive(message);
    }

}

/// Helper function for running a blocking actor.
pub (super) fn run_a_blocking_actor<R, T: Actor<R>>(mut actor: BlockingActorImpl<T, R>) {
    while let Ok(message) = actor.receiver.recv() {
        actor.receive(message);
    }
}


/// Implements an actor with an Actor type T, and a Payload type R (can be the NotUsed type if no payload is received).
pub (super) struct ActorImpl<T, R> {
    name: Option<String>,
    actor: T,
    receiver: mpsc::Receiver<Message<ActorType, R>>,
}

impl <R, T: Actor<R>> ActorImpl <T, R> {
    pub (super) fn receive(&mut self, message: Message<ActorType, R>){
        info!("Actor {} received message", self.name.clone().unwrap_or("Unnamed Actor".to_string()));
        T::receive(&mut self.actor, message);
    }
    pub (super) fn new (name: Option<String>, actor: T, receiver: mpsc::Receiver::<Message<ActorType, R>>) -> Self {

        ActorImpl {name, actor, receiver }
    }
}

pub (super) struct BlockingActorImpl<T, R> {
    name: Option<String>,
    actor: T,
    receiver: std::sync::mpsc::Receiver<Message<ActorType, R>>,
}

impl <R, T: Actor<R>> BlockingActorImpl<T, R> {

     pub (super) fn receive(&mut self, message: Message<ActorType, R>){ 
        info!("Blocking actor {} received message", self.name.clone().unwrap_or("Unnamed Blocking Actor".to_string()));
        T::receive(&mut self.actor, message);
    }

    pub (super) fn new (name: Option<String>, actor: T, receiver: std::sync::mpsc::Receiver<Message<ActorType, R>>) -> BlockingActorImpl<T, R> {
        BlockingActorImpl { name, actor, receiver}
    }
}