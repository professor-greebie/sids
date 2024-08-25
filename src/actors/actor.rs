use super::{actor_ref::ActorRef, guardian::Guardian, messages::InternalMessage};
use log::info;
use tokio::sync::mpsc;

struct Dummy;
impl ActorTrait for Dummy {
    fn receive(&mut self, _message: InternalMessage) {
        // do nothing
    }
}


pub (super) fn create_dummy_actor() -> ActorRef {
    let (tx, rx) = mpsc::channel(1);
    let actor = Actor::new(None, Dummy, rx);
    let actor_ref = ActorRef::new(actor, tx);
    actor_ref
}


/// The main actor trait that all actors must implement.
#[trait_variant::make(Send)]
pub trait ActorTrait{
    fn receive(&mut self, message: InternalMessage) where Self: Sized;
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
pub (super) async fn run_an_actor<T: ActorTrait + 'static>(mut actor : Actor<T>) {
    while let Some(message) = actor.receiver.recv().await {
        info!("Running an actor");
        actor.receive(message);
    }

}

/// Helper function for running a blocking actor.
pub (super) fn run_a_blocking_actor<T: ActorTrait>(mut actor: BlockingActor<T>) {
    while let Ok(message) = actor.receiver.recv() {
        actor.receive(message);
    }
}


pub (super) struct Actor<T> {
    name: Option<String>,
    actor: T,
    receiver: mpsc::Receiver<InternalMessage>,
}

impl <T: ActorTrait> Actor<T> {
    pub (super) fn receive(&mut self, message: InternalMessage){
        info!("Actor {} received message", self.name.clone().unwrap_or("Unnamed Actor".to_string()));
        ActorTrait::receive(&mut self.actor, message);
    }
    pub (super) fn new (name: Option<String>, actor: T, receiver: mpsc::Receiver::<InternalMessage>) -> Self {

        Actor {name, actor, receiver }
    }
}

pub (super) struct BlockingActor<T> {
    name: Option<String>,
    actor: T,
    receiver: std::sync::mpsc::Receiver<InternalMessage>
}

impl <T: ActorTrait> BlockingActor<T> {
    #[allow(dead_code)]
     pub (super) fn receive(&mut self, message: InternalMessage) {
        info!("Blocking actor {} received message", self.name.clone().unwrap_or("Unnamed Blocking Actor".to_string()));
        ActorTrait::receive(&mut self.actor, message);
    }

    pub (super) fn new (name: Option<String>, actor: T, receiver: std::sync::mpsc::Receiver<InternalMessage>) -> BlockingActor<T> {
        BlockingActor { name, actor, receiver}
    }
}