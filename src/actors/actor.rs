use std::sync::mpsc;
use crate::actors::actor_ref::ActorRef;

pub trait ActorMessage {

}

pub enum GuardianMessage {
    Terminated,
    StopChild(ActorRef),
}

pub trait A {

}


pub trait Actor<T> : A {
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

impl A for Guardian {}

impl Actor<GuardianMessage> for Guardian {
    fn receive(&mut self, message: mpsc::Receiver<GuardianMessage>) {
        loop {
            match message.recv() {
                Ok(GuardianMessage::Terminated) => {
                    println!("Terminating actor");
                    break;
                }
                Ok(GuardianMessage::StopChild(actor)) => {
                    println!("Stopping child actor");
                    actor.stop();
                }
                Err(_) => {
                    println!("Error receiving message");
                    break;
                }
            }
        }
    }

}



pub struct Source<T, ActorRef> {
    next: Option<T>,
    payload: T,
    virtualizer: ActorRef,
    
}







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor() {
    }
}
