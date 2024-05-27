use std::time::Duration;
use std::sync::mpsc;
use crate::actors::actor_ref::ActorRef;

pub trait ActorMessage {

}

pub enum GuardianMessage {
    Terminated,
    StopChild(ActorRef),
}


pub trait Actor<T> {
    fn receive(&mut self, message: mpsc::Receiver<T>);
    fn name(&self) -> &str {
        "Unnamed Actor"
    }
    fn next_actor(&self) -> Option<ActorRef> {
        None
    }
}

pub struct Guardian {

}

impl Actor<GuardianMessage> for Guardian {
    fn receive(&mut self, message: mpsc::Receiver<GuardianMessage>) {
        loop {
            match message.recv() {
                GuardianMessage::Terminate => {
                    println!("Terminating actor");
                    break;
                }
                GuardianMessage::StopChild(actor) => {
                    println!("Stopping child actor");
                    actor.stop();
                }
            }
        }
    }

}



pub struct Source<T, ActorRef> {
    next: Option<T>,
    payload: T,
    virtualizer: ActorRef,
    
};

impl Actor<T> for Source<T, ActorRef> {


    fn receive(&mut self, message: mpsc::Receiver<T>) {
        loop {
            match message.recv() {
                Ok(msg) => {
                    println!("Received message: {:?}", msg);
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
    }
    
    fn name(&self) -> &str {
        "Unnamed Actor"
    }
    
    fn next_actor(&self) -> Option<ActorRef> {
        None
    }

}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor() {
        let (sender, receiver) = mpsc::channel();
        let mut actor = GetResourceActor {
            receiver,
            next_actor: None,
        };
        actor.receive(receiver);
    }
}
