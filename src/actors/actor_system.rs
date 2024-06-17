use crate::actors::actor::Actor;
use crate::actors::actor_ref::ActorRef;
use super::actor::{GetActor, Guardian, SelectActor};

// An actor system is a collection of actors that can communicate with each other.
pub struct ActorSystem {
    // The current actor reference.
    _value: Option<ActorRef>,
    _actors: Option<Box<ActorSystem>>,
}



impl ActorSystem 
 {

    pub fn new () -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(1);
        let actor = Actor::Guardian(Guardian::new(rec));
        let actor_ref = ActorRef::new(actor, snd);
        ActorSystem { _value: Some(actor_ref), _actors: None }
    }

    pub fn spawn_actor(&mut self, actor_select: SelectActor) {
        let (snd, rec) = tokio::sync::mpsc::channel(1);
        match actor_select {
            SelectActor::GetActor => {
                let actor = Actor::GetActor(GetActor::new(rec));
                let actor_ref = ActorRef::new(actor, snd);
                self._value = Some(actor_ref);
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
    use super::*;

    #[tokio::test]
    async fn test_actor_system() {
        let mut actor_system = ActorSystem::new();
         actor_system.spawn_actor(SelectActor::GetActor);
        assert_eq!(actor_system.next_actor().is_some(), true);
        assert_eq!(actor_system._value.is_some(), true);
        assert_eq!(actor_system._actors.is_some(), true);
    }
}