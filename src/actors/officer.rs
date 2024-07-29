use super::{actor_ref, messages};
use super::actor_ref::ActorRef;
use super::messages::InternalMessage;

#[derive(Debug, Clone, Copy)]
pub enum SelectActor {
    Guardian,
    LogActor,
    Collector,
    CleaningActor,
    KafkaProducerActor,
}
pub (super) struct Officer { 
    pub (super) _id: u32,
    pub (super) _type: SelectActor,
    pub (super) actor: ActorRef,
    pub (super) courriers: Vec<actor_ref::ActorRef>,    
}

impl Officer{

    pub async fn send(&mut self, message: messages::Message) {
        // ought the officer act as an ActorRef or should it have an ActorRef itself
        // or do we always pass a Message to the ActorRef and let the actor ref convert it?
        match self.actor {
            actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                blocking_actor_ref.send(&message.to_internal_message(None).unwrap());
            },
            actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                tokio_actor_ref.send(&message.to_internal_message(None).unwrap()).await;
            }
        }
    }


    #[allow(dead_code)]
    pub fn subscribe(&mut self, actor: actor_ref::ActorRef) {
        self.courriers.push(actor);
    }
    #[allow(dead_code)]
    pub fn unsubscribe(&mut self, actor_id: u32) {
        self.courriers.remove(actor_id as usize);
    }

    #[allow(dead_code)]
    pub async fn notify(&mut self, message: &InternalMessage) -> Result<(), std::io::Error> {
        self.notify_blocking_courriers(message);
        self.notify_tokio_courriers(message).await;
        Ok(())
    }

    pub fn notify_blocking_courriers(&mut self, message: &InternalMessage) {
        for courier in self.courriers.iter_mut() {
            match courier {
                actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                    blocking_actor_ref.send(message);
                },
                _ => {}
            }
        }
    }

    pub async fn notify_tokio_courriers(&mut self, message: &messages::InternalMessage) {
        for courier in self.courriers.iter_mut() {
            match courier {
                actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                    tokio_actor_ref.send(message).await;
                },
                _ => {}
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::actors::actor::{Actor, CleaningActor, Collector};

    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_officer() {
        let (_tx, rx) = mpsc::channel(1);
        let (_tx2, rx2) = mpsc::channel(1);
        let mut officer = Officer {
            _id: 1,
            _type: SelectActor::Collector,
            actor: actor_ref::ActorRef::TokioActorRef(actor_ref::TokioActorRef::new(Actor::CleaningActor(CleaningActor::new(rx)), _tx).expect("Failed to create actor ref.")),
            courriers: Vec::new(),
        };
        let actor_ref = actor_ref::ActorRef::TokioActorRef(actor_ref::TokioActorRef::new(Actor::CleaningActor(CleaningActor::new(rx2)), _tx2).expect("Failed to create actor ref."));
        officer.subscribe(actor_ref);
        assert!(officer.courriers.len() == 1);
        officer.notify(&messages::InternalMessage::Terminate).await.expect("Failed to notify courriers.");
    }
    #[tokio::test]

    async fn test_blocking_officer() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let actor_ref = actor_ref::ActorRef::BlockingActorRef(actor_ref::BlockingActorRef::new(Actor::Collector(Collector::new(rx)), _tx).expect("Failed to create actor ref."));
        let mut officer = Officer {
            _id: 1,
            _type: SelectActor::Collector,
            actor: actor_ref,
            courriers: Vec::new(),
        };
        let (_tx2, rx2) = std::sync::mpsc::channel();
        let (tx3, rx3) = tokio::sync::mpsc::channel(10);
        let actor_ref = actor_ref::ActorRef::BlockingActorRef(actor_ref::BlockingActorRef::new(Actor::Collector(Collector::new(rx2)), _tx2).expect("Failed to create actor ref."));
        officer.subscribe(actor_ref);
        assert!(officer.courriers.len() == 1);
        let actor_ref_tokio = actor_ref::ActorRef::TokioActorRef(actor_ref::TokioActorRef::new(Actor::CleaningActor(CleaningActor::new(rx3)), tx3).expect("Failed to create actor ref."));
        officer.subscribe(actor_ref_tokio);
        assert!(officer.courriers.len() == 2);
        officer.notify(&messages::InternalMessage::Terminate).await.expect("Failed to notify courriers.");
        
    }

}
