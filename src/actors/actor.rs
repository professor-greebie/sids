use crate::actors::messages::{ActorMessage, GuardianMessage};
use std::sync::mpsc;

pub trait Actor {
    type T: ActorMessage;

    fn receive(&self, message: mpsc::Receiver<Self::T>);
    fn name(&self) -> &str {
        "Unnamed Actor"
    }
}

#[derive(Debug, Clone)]
pub struct Guardian {}

impl Actor for Guardian {
    type T = GuardianMessage;
    fn receive(&self, message: mpsc::Receiver<Self::T>) -> () {
        loop {
            match message.recv() {
                Ok(GuardianMessage::Terminated) => {
                    println!("Terminating actor");
                    break;
                }
                Err(e) => {
                    println!("Error receiving message: {:?}", e);
                    break;
                }
            }

        }
}}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor() {
        let (sender, receiver) = mpsc::channel();
        sender.send(GuardianMessage::Terminated).unwrap();
        let actor = Guardian {};
        actor.receive(receiver);
    }
}
