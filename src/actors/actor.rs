use std::time::Duration;
use std::sync::mpsc;

pub enum ActorMessage {
    SayHello,
    SayGoodbye,
}

pub trait Actor<T> {
    fn receive(&mut self, message: mpsc::Receiver<T>);
    fn name(&self) -> &str {
        "Unnamed Actor"
    }
    fn next_actor(&self) -> Option<String> {
        None
    }
}

struct Actor {
    receiver : mpsc::Receiver<ActorMessage>,
    next_actor: Option<String>,
}

impl Actor<ActorMessage> for GetResourceActor {
    fn receive(&mut self, message: mpsc::Receiver<ActorMessage>) {
        loop {
            let message = message.recv().unwrap();
            match message {
                ActorMessage::SayHello => println!("Hello!"),
                ActorMessage::SayGoodbye => println!("Goodbye!"),
            }
        }
    }

    fn name(&self) -> &str {
        "GetResourceActor"
    }
}

pub struct GetResourceActorRef {
    sender: mpsc::Sender<ActorMessage>,
}

pub trait ActorRef<T> {
    fn send(&self, message: mpsc::Sender<T>);
    fn actor_name(&self) -> &str;
}

impl ActorRef<ActorMessage> for GetResourceActorRef {
    fn send(&self, message: mpsc::Sender<ActorMessage>) {
        message.send(ActorMessage::SayHello).unwrap();
    }

    fn actor_name(&self) -> &str {
        "GetResourceActor"
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
