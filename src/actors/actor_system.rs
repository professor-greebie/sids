
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;


const DEFAULT_ACTOR_STSTEM_SPAWN_SIZE: usize = 100;

trait ActorRefFactory {
    /// Create an actor and return a reference to it.
    fn actor_of(props: Props) -> ActorRef;
    /// Reference to guardian actor, the topmost actor in the actor hierarchy.
    fn guardian() -> ActorRef;
    /// Stop the actor pointed to by the ActorRef.
    fn stop() -> ();
}

use super::actor::{Actor, ActorRef};
/// An actor system is a collection of actors that can communicate with each other.
struct ActorSystem {
    actors: HashMap<ActorRef, thread::JoinHandle<()>>,
    name: String,
    config: Option<HashMap<String, String>>,
}

impl ActorRefFactory for ActorSystem {
    fn actor_of(props: Props) -> ActorRef {
        todo!(); // todo: Create an id for the actor and reference.
        todo!(); // Spawn an actor thread based on the Props provided.
        todo!(); // Return the ActorReference.
        unimplemented!("Implement the actor_of method");
    }
    fn guardian() -> ActorRef {
        todo!(); // return reference to topmost actor in the actor hierarchy.
    }
    fn stop() -> () {
        todo!(); // stop the actor pointed to by the ActorRef.
    }
}

// establish an ActorRef system

impl ActorSystem {
    fn new() -> ActorSystem {
        ActorSystem {
            actors: HashMap::new(),
            name: "default".to_string(),
            config: None,
        }
    }
    fn with_configuration(&mut self, config: HashMap<String, String>) -> &mut ActorSystem {
        self.name = "default".to_string();
        self.config = Some(config);
        self
    }

    fn spawn<A, M>(&mut self, actor_ref: ActorRef)
    where
        A: Actor<M> + 'static,
        M: Send + 'static,
    {
        // Create the actor reference.
        let spawn_size = DEFAULT_ACTOR_STSTEM_SPAWN_SIZE;
        let (sender, receiver) = channel();
        let actor = Arc::new(Mutex::new(sender));
        let handle = thread::spawn(move || {
             // Add this line
            loop {
                let sender = actor.lock(); // Fix the type parameter
                match sender  {
                    Ok(sender) => {
                        sender.send(actor_ref);
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                } 
                
             receiver.recv().unwrap();
        }});
        self.actors.insert(actor_ref.name().to_owned(), handle);
        
    }
}

#[cfg(test)]
mod tests {
    use crate::actors::actor::TimedActor;

    use super::*;

    #[test]
    fn test_actor_system() {
        let mut system = ActorSystem::new();
        let actor = TimedActor::new("test_actor");
        system.spawn(actor);
        assert_eq!(system.actors.len(), 1);
    }
}