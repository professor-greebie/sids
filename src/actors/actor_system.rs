use crate::actors::actor::Actor;
use crate::actors::actor_ref::{ActorRef, GetActorRef};
use super::actor::ActorType;

enum ActorSystemTypes {
    ActorRef(ActorRef), 
    GetActorRef(ActorRef)
}

// An actor system is a collection of actors that can communicate with each other.
pub struct ActorSystem {
    // The actor system has a collection of actors that can communicate with each other.
    _value: Option<ActorSystemTypes>,
    _actors: Option<Box<ActorSystem>>,
}



impl ActorSystem 
 {

    pub fn new () -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(1);
        let actor = Actor::new(rec);
        let actor_ref = ActorRef::new(actor, snd);
        ActorSystem { _value: Some(ActorSystemTypes::ActorRef(actor_ref)), _actors: None }
    }

    pub fn spawn_actor(&mut self) {
        
        let (snd, rec) = tokio::sync::mpsc::channel(1);
        let actor = ActorType::new(rec);
        let actor_ref = ActorRef::new(actor, snd);
        self._actors = Some(Box::new(ActorSystem { _value: Some(ActorSystemTypes::ActorRef(actor_ref)), _actors: None }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_actor_system() {
        let actor_system = ActorSystem::new();
        assert_eq!(actor_system._value.is_some(), true);
    }
}