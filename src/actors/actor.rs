use super::{actor_ref::ActorRef, guardian::Guardian, messages::Message};
use log::info;
use tokio::sync::mpsc;

struct Dummy;
impl Actor for Dummy {
    fn receive(&mut self, _message: Message) {
        // do nothing
    }
}


pub (super) fn create_dummy_actor() -> ActorRef {
    let (tx, rx) = mpsc::channel(1);
    let actor = ActorImpl::new(None, Dummy, rx);
    ActorRef::new(actor, tx)
}


/// The main actor trait that all actors must implement.
#[trait_variant::make(Send)]
pub trait Actor{
    fn receive(&mut self, message: Message) where Self: Sized;
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
pub (super) async fn run_an_actor<T: Actor + 'static>(mut actor : ActorImpl <T>) {
    while let Some(message) = actor.receiver.recv().await {
        info!("Running an actor");
        actor.receive(message);
    }

}

/// Helper function for running a blocking actor.
pub (super) fn run_a_blocking_actor<T: Actor>(mut actor: BlockingActorImpl<T>) {
    while let Ok(message) = actor.receiver.recv() {
        actor.receive(message);
    }
}


pub (super) struct ActorImpl<T> {
    name: Option<String>,
    actor: T,
    receiver: mpsc::Receiver<Message>,
}

impl <T: Actor> ActorImpl<T> {
    pub (super) fn receive(&mut self, message: Message){
        info!("Actor {} received message", self.name.clone().unwrap_or("Unnamed Actor".to_string()));
        Actor::receive(&mut self.actor, message);
    }
    pub (super) fn new (name: Option<String>, actor: T, receiver: mpsc::Receiver::<Message>) -> Self {

        ActorImpl {name, actor, receiver }
    }
}

pub (super) struct BlockingActorImpl<T> {
    name: Option<String>,
    actor: T,
    receiver: std::sync::mpsc::Receiver<Message>
}

impl <T: Actor> BlockingActorImpl<T> {
    #[allow(dead_code)]
     pub (super) fn receive(&mut self, message: Message) {
        info!("Blocking actor {} received message", self.name.clone().unwrap_or("Unnamed Blocking Actor".to_string()));
        Actor::receive(&mut self.actor, message);
    }

    pub (super) fn new (name: Option<String>, actor: T, receiver: std::sync::mpsc::Receiver<Message>) -> BlockingActorImpl<T> {
        BlockingActorImpl { name, actor, receiver}
    }
}