use crate::actors::actor;
use crate::actors::messages;
use crate::actors::actor_ref;


pub struct ActorSystem {
    
    head: officer::Link // the current actor reference. Should never be headless.
}


impl ActorSystem {
    pub fn new() -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(4);
        let actor = actor::Actor::Guardian(actor::Guardian::new(rec));
        let actor_ref = actor_ref::ActorRef::TokioActorRef(actor_ref::TokioActorRef::new(actor, snd));
        ActorSystem {
            head : officer::Link::More(Box::new(officer::Officer {
                _id: 0,
                _type: officer::SelectActor::Guardian,
                actor: actor_ref,
                courriers: Vec::new(),
                next: officer::Link::Empty,
            })),
        }
    }

    pub async fn dispatch(self, message: &messages::Message) {
        if let officer::Link::More(mut officer) = self.head {
            officer.send(message).await;
        }
    }

    pub async fn notify_courriers(self, message: &messages::Message) {
        if let officer::Link::More(mut officer) = self.head {
            officer.notify_courriers(message);
            
        }
    }

}

mod officer {

    use crate::actors::actor_ref::BlockingActorRef;

    use super::actor_ref;
    use super::messages;
    use super::actor;
    pub(super) enum SelectActor {
        Guardian,
        LogActor,
        Collector,
        KafkaProducerActor,
    }
    
    impl SelectActor {
        pub fn get_actor_description(&self) -> String {
            match self {
                SelectActor::Guardian => "Guardian Actor".to_string(),
                SelectActor::LogActor => "Log Actor".to_string(),
                SelectActor::Collector => "Collector Actor".to_string(),
                SelectActor::KafkaProducerActor => "Kafka Producer Actor".to_string(),
            }
        }
    }
    
    pub (super) enum Link {
        Empty,
        More(Box<Officer>),
    }
    pub (super) struct Officer { 
        pub (super) _id: u32,
        pub (super) _type: SelectActor,
        pub (super) actor: actor_ref::ActorRef,
        pub (super) courriers: Vec<actor_ref::ActorRef>,
        pub (super) next: Link,
    }
    
    impl Officer {
    
        pub fn new(id: u32, actor: actor_ref::ActorRef, next: Link) -> Self {
            Officer {
                _id: id,
                _type: SelectActor::Guardian,
                actor: actor,
                courriers: Vec::new(),
                next: next,
            }
        }
    
        pub async fn send(&mut self, message: &messages::Message) {
            match self.actor {
                actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                    blocking_actor_ref.send(message);
                },
                actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                    tokio_actor_ref.send(message).await;
                }
            }
        }
    
        pub fn subscribe(&mut self, actor: actor_ref::ActorRef) {
            self.courriers.push(actor);
        }
    
        pub fn unsubscribe(&mut self, actor: actor_ref::ActorRef) {
            //self.courriers.retain(|a| a != &actor);
        }
    
        pub fn notify_courriers(&mut self, message: &messages::Message) {
            for courier in self.courriers.iter_mut() {
                match courier {
                    actor_ref::ActorRef::BlockingActorRef(ref mut blocking_actor_ref) => {
                        blocking_actor_ref.send(message);
                    },
                    actor_ref::ActorRef::TokioActorRef(ref mut tokio_actor_ref) => {
                        tokio_actor_ref.send(message);
                    }
                }
            }
        }
    
        pub fn spawn_officer(&mut self, actor_select: SelectActor) {
            match actor_select {
                SelectActor::Collector => {
                    let (snd, rec) = std::sync::mpsc::channel::<messages::Message>();
                    let collector = actor::Collector::new(rec);
                    let actor = actor::Actor::Collector(collector);
                    let actor_ref = actor_ref::ActorRef::BlockingActorRef(BlockingActorRef::new(actor, snd));
                    
                    match &self.next {
                            Link::Empty => {
                                let new_officer = Officer::new(self._id + 1, actor_ref, Link::Empty);
                                self.next = Link::More(Box::new(new_officer))
                            },
                            Link::More(officer) => {
    
                                // Perhaps it's better to prepend collectors to the head of the list?
                                let mut next = &officer.next;
                                let mut id = officer._id;
                                while let Link::More(next_officer) = next {
                                    id = next_officer._id;
                                    next = &next_officer.next;
                                }
                                let new_officer = Officer::new(id + 1, actor_ref, Link::Empty);
                                next = &Link::More(Box::new(new_officer));
                        }
                    }
                },
                SelectActor::KafkaProducerActor => {
                    let (snd, rec) = tokio::sync::mpsc::channel::<messages::Message>(32);
                    let actor = actor::Actor::KafkaProducerActor(actor::KafkaProducerActor::new(rec));
                    let actor_ref = actor_ref::ActorRef::TokioActorRef(actor_ref::TokioActorRef::new(actor, snd));
                    
                    match &self.next {
                        Link::Empty => {
                            let new_officer = Officer::new(self._id + 1, actor_ref, Link::Empty);
                            self.next = Link::More(Box::new(new_officer))}
                            ,
                        Link::More(officer) => {
    
                            // Perhaps it's better to prepend collectors to the head of the list?
                            let mut next = &officer.next;
                            let mut id = officer._id;
                            while let Link::More(next_officer) = next {
                                id = next_officer._id;
                                next = &next_officer.next;
                            }
                            let new_officer = Officer::new(id + 1, actor_ref, Link::Empty);
                            next = &Link::More(Box::new(new_officer));
                    }
                }
                },
                _ => {}
            }
        }
    
    }
}