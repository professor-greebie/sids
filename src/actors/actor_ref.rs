use crate::actors::actor::Actor;
use super::actor::{ActorType, GetActor, Guardian};
use super::messages::{ActorMessage, GetActorMessage, Message};


trait ActorRefTrait {
    fn send(&mut self, message: Message);
}

pub struct ActorRef {
    sender: tokio::sync::mpsc::Sender<Message>,
}

impl ActorRef {

    pub fn new(actor: Actor, snd: tokio::sync::mpsc::Sender<Message>) -> Self {
        match actor {
            Actor::Guardian(mut guardian) => {
                tokio::spawn(async move {
                    guardian.run().await;
                });
            },
            Actor::GetActor(mut get_actor) => {
                tokio::spawn(async move {
                    get_actor.run().await;
                });
            },
            _ => {}
        }
        Self{ sender: snd }
    }

    pub async fn send(&mut self, message: Message) {
        self.sender.send(message).await.unwrap();
    }
}



