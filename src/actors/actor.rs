use crate::actors::messages::{ActorMessage, GuardianMessage};
use tokio::sync::mpsc;

pub trait Actor {
    type T: ActorMessage;
    fn new(receiver: mpsc::Receiver<Self::T> ) -> Self;
    fn receive(&mut self, message: Self::T);
    fn name(&self) -> &str {
        "Unnamed Actor"
    }
}


pub struct Guardian {
    pub receiver: mpsc::Receiver<GuardianMessage>,
    next_id: u64,
    name: String
}

impl Actor for Guardian {
    type T = GuardianMessage;
    fn new (receiver: mpsc::Receiver<GuardianMessage>) -> Self {
        Guardian {
            receiver,
            next_id: 0,
            name: "Guardian".to_string()
        }
    }

    fn receive(&mut self, message: Self::T) {
        match message {
            GuardianMessage::Terminated => {
                println!("{} received a termination message", self.name);
            }
            GuardianMessage::GetNextId { responder } => {
                self.next_id += 1;
                let _ = responder.send(self.next_id);
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor() {
        let (sender, receiver) = mpsc::channel(1);
        sender.send(GuardianMessage::Terminated);
        let actor = Guardian::new(receiver);
    }
}
