use super::{guardian::Guardian, messages::InternalMessage};
use tokio::sync::mpsc;


// Will attempt to use this to abstract over ActorTrait and BlockingActorTrait at some point


pub (super) async fn blocking_actor_receive( actor: &mut dyn BlockingActorTrait, message: InternalMessage) {
    actor.handle_message(message);
}

#[trait_variant::make(ActorTrait: Send)]
pub trait ActorTraitImpl{
    async fn receive(&mut self, message: InternalMessage) where Self: Sized;
}

pub trait BlockingActorTrait: ActorTrait {
    async fn receive(&mut self, message: InternalMessage) where Self: Sized {
        blocking_actor_receive(self, message).await;
    }

    fn handle_message(&mut self, message: InternalMessage);
}

pub (super) async fn run_guardian(mut actor: Guardian) {
    while let Some(message) = actor.receiver.recv().await {
        actor.receive(message).await;
    }
}

pub (super) async fn run_an_actor<T: ActorTrait>(mut actor : Actor<T>) {
    while let Some(message) = actor.receiver.recv().await {
        actor.receive(message).await;
    }

}

pub (super) fn run_a_blocking_actor<T: BlockingActorTrait>(mut actor: BlockingActor<T>) {
    while let Ok(message) = actor.receiver.recv() {
        actor.receive(message);
    }
}


pub (super) struct Actor<T> {
    actor: T,
    receiver: mpsc::Receiver<InternalMessage>,
}

impl <T: ActorTrait> Actor<T> {
    pub (super) async fn receive(&mut self, message: InternalMessage){
        ActorTrait::receive(&mut self.actor, message).await;
    }
    pub (super) fn new (actor: T, receiver: mpsc::Receiver::<InternalMessage>) -> Self {

        Actor { actor, receiver }
    }
}

pub (super) struct BlockingActor<T> {
    actor: T,
    receiver: std::sync::mpsc::Receiver<InternalMessage>
}

impl <T: BlockingActorTrait> BlockingActor<T> {
    pub (super) fn receive(&mut self, message: InternalMessage) {
        BlockingActorTrait::receive(&mut self.actor, message);
    }
    pub (super) fn new (actor: T, receiver: std::sync::mpsc::Receiver<InternalMessage>) -> BlockingActor<T> {
        BlockingActor { actor, receiver}
    }
}