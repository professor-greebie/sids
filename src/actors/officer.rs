use log::info;

use super::actor_ref::ActorRef;
use super::messages::InternalMessage;
use super::{actor_ref, messages};

#[derive(Debug, Clone, Copy)]

/// The list of acceptable actor types included in the Actor System.
///
/// The `SelectActor` enum is used to specify the type of actor that the officer will be responsible for.
/// It is also used for courriers. The list can expand to other types as required.
/// In the future, we may see another way of specifying the type of actor, but for now, this is the best way to do it.
pub enum SelectActor {
    Guardian,
    LogActor,
    Collector,
    CleaningActor,
    KafkaProducerActor,
    KafkaConsumerActor,
}

/// The Officer struct is an actor that controls a number of courriers in an actor system.
///
/// The intention of the officer is multi-faceted. It can be used to control the state of the courriers,
/// and also to send messages to the courriers. The officer is also an actor, and can be used to send messages to itself.
/// Officers will be controlled by the guardian actor, which is the main actor in the system.
pub(super) struct Officer {
    pub(super) _id: u32,
    pub(super) _type: SelectActor,
    pub(super) actor: ActorRef,
    pub(super) courriers: Vec<actor_ref::ActorRef>,
}

impl Officer {
    pub async fn send(&mut self, message: messages::Message) {
        info!(actor="officer"; "Sending message to officer {}.", self._id);

        match message {
            messages::Message::Terminate => {
                self.notify(&messages::InternalMessage::Terminate)
                    .await
                    .expect("Failed to notify courriers.");
                match self.actor {
                    actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                        blocking_actor_ref.send(&messages::InternalMessage::Terminate);
                    }
                    actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                        tokio_actor_ref
                            .send(&messages::InternalMessage::Terminate)
                            .await;
                    }
                }
            }
            messages::Message::GetURI { uri, location } => match self.actor {
                actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                    blocking_actor_ref.send(&messages::InternalMessage::CollectorMessage(
                        messages::CollectorMessage::GetURITemplate {
                            uri: uri,
                            location: location,
                        },
                    ));
                }
                _ => {}
            },
            _ => match self.actor {
                actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                    blocking_actor_ref.send(&message.to_internal_message(None).unwrap());
                }
                actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                    tokio_actor_ref
                        .send(&message.to_internal_message(None).unwrap())
                        .await;
                }
            },
        }
    }

    /// Add a courrier to the officer's list of courriers.
    pub fn subscribe(&mut self, actor: actor_ref::ActorRef) {
        self.courriers.push(actor);
    }
    /// Remove a courrier from the officer's list of courriers.
    pub fn unsubscribe(&mut self, actor_id: u32) {
        self.courriers.remove(actor_id as usize);
    }
    /// Send a message to all courriers.
    pub async fn notify(&mut self, message: &InternalMessage) -> Result<(), std::io::Error> {
        self.notify_blocking_courriers(message);
        self.notify_tokio_courriers(message).await;
        Ok(())
    }
    /// Send a message to all blocking courriers.
    pub fn notify_blocking_courriers(&mut self, message: &InternalMessage) {
        for courier in self.courriers.iter_mut() {
            match courier {
                actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                    blocking_actor_ref.send(message);
                }
                _ => {}
            }
        }
    }
    /// Send a message to all tokio (async)courriers.
    pub async fn notify_tokio_courriers(&mut self, message: &messages::InternalMessage) {
        for courier in self.courriers.iter_mut() {
            match courier {
                actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                    tokio_actor_ref.send(message).await;
                }
                _ => {}
            }
        }
    }
}

// grcov-excl-start
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
            actor: actor_ref::ActorRef::TokioActorRef(
                actor_ref::TokioActorRef::new(Actor::CleaningActor(CleaningActor::new(rx)), _tx)
                    .expect("Failed to create actor ref."),
            ),
            courriers: Vec::new(),
        };
        let actor_ref = actor_ref::ActorRef::TokioActorRef(
            actor_ref::TokioActorRef::new(Actor::CleaningActor(CleaningActor::new(rx2)), _tx2)
                .expect("Failed to create actor ref."),
        );
        officer.subscribe(actor_ref);
        assert!(officer.courriers.len() == 1);
        officer
            .notify(&messages::InternalMessage::Terminate)
            .await
            .expect("Failed to notify courriers.");
    }
    #[tokio::test]

    async fn test_blocking_officer() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let actor_ref = actor_ref::ActorRef::BlockingActorRef(
            actor_ref::BlockingActorRef::new(Actor::Collector(Collector::new(rx)), _tx)
                .expect("Failed to create actor ref."),
        );
        let mut officer = Officer {
            _id: 1,
            _type: SelectActor::Collector,
            actor: actor_ref,
            courriers: Vec::new(),
        };
        let (_tx2, rx2) = std::sync::mpsc::channel();
        let (tx3, rx3) = tokio::sync::mpsc::channel(10);
        let actor_ref = actor_ref::ActorRef::BlockingActorRef(
            actor_ref::BlockingActorRef::new(Actor::Collector(Collector::new(rx2)), _tx2)
                .expect("Failed to create actor ref."),
        );
        officer.subscribe(actor_ref);
        assert!(officer.courriers.len() == 1);
        let actor_ref_tokio = actor_ref::ActorRef::TokioActorRef(
            actor_ref::TokioActorRef::new(Actor::CleaningActor(CleaningActor::new(rx3)), tx3)
                .expect("Failed to create actor ref."),
        );
        officer.subscribe(actor_ref_tokio);
        assert!(officer.courriers.len() == 2);
        officer
            .notify(&messages::InternalMessage::Terminate)
            .await
            .expect("Failed to notify courriers.");
    }
}

// grcov-excl-stop
