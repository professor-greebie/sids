use crate::actors::actor::Actor;
use crate::actors::actor_ref::{ActorRef, SenderType};
use super::actor::{GetActor, Guardian, KafkaProducerActor, SelectActor};
use super::messages::Message;

// An actor system is a collection of actors that can communicate with each other.
pub struct ActorSystem {
    // The current actor reference.
    pub _value: Option<ActorRef>,
    pub _actors: Option<Box<ActorSystem>>,
}



impl ActorSystem 
 {

    pub fn new () -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(4);
        let sender = SenderType::TokioSender(snd);
        let actor = Actor::Guardian(Guardian::new(rec));
        let actor_ref = ActorRef::new(actor, sender);
        ActorSystem { _value: Some(actor_ref), _actors: None }
    }

    pub fn spawn_actor(&mut self, actor_select: SelectActor) {
        
        match actor_select {
            SelectActor::GetActor => {
                let (snd, rec) = std::sync::mpsc::channel::<Message>();
                let sender = SenderType::StdSender(snd);
                let actor = Actor::GetActor(GetActor::new(rec));
                let actor_ref = ActorRef::new(actor, sender);
                self._actors = Some(Box::new(ActorSystem {_value: Some(actor_ref), _actors: None}));
            },
            SelectActor::KafkaProducerActor => {
                let (snd, rec) = tokio::sync::mpsc::channel::<Message>(32);
                let sender = SenderType::TokioSender(snd);
                let actor = Actor::KafkaProducerActor(KafkaProducerActor::new(rec));
                let actor_ref = ActorRef::new(actor, sender);
                self._actors = Some(Box::new(ActorSystem {_value: Some(actor_ref), _actors: None}));

            },
            _ => {}
        }
        
    }

    pub fn next_actor(&self) -> Option<&Box<ActorSystem>> {
        self._actors.as_ref()
    }
}

#[cfg(test)]
mod tests {


    #[tokio::test]
    async fn test_actor_system() {

    }

    #[tokio::test]
    async fn test_send_message() {
        
    }
}