use crate::actors::actor::Actor;
use super::actor::{ActorType, GetActor};
use super::messages::{ActorMessage, GetActorMessage};

pub struct ActorRef {
    sender: tokio::sync::mpsc::Sender<Box<ActorMessage>>,
}

impl ActorRef {

    pub fn new(mut actor: Actor, snd: tokio::sync::mpsc::Sender<Box<ActorMessage>>) -> Self {
        tokio::spawn(async move {
            actor.run().await;
        });
        Self{ sender: snd }
    }

    pub async fn send(&mut self, message: Box<ActorMessage>) {
        self.sender.send(message).await.unwrap();
    }
}

pub struct GetActorRef {
    sender: tokio::sync::mpsc::Sender<Box<GetActorMessage>>,
}

impl GetActorRef {

    pub fn new(mut actor: GetActor, snd: tokio::sync::mpsc::Sender<Box<GetActorMessage>>) -> Self {
        tokio::spawn(async move {
            actor.run().await;
        });
        Self{ sender: snd }
    }

    pub async fn send(&mut self, message: Box<GetActorMessage>) {
        self.sender.send(message).await.unwrap();
    }
}


