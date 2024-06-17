use tokio::sync::mpsc;

use super::messages::{ActorMessage, GetActorMessage, Message};


pub trait ActorType {
    fn receive(&self, message : Message);
}

pub enum Actor {
    Guardian(Guardian),
    GetActor(GetActor),
}

pub enum SelectActor {
    Guardian,
    GetActor,
}

pub struct Guardian {
    pub receiver: mpsc::Receiver<Message> ,
}

impl ActorType for Guardian {
    fn receive(&self, message: Message) {
        match message {
            Message::ActorMessage(ActorMessage::Terminate) => {
                println!("Actor terminated");
            },
            Message::ActorMessage(ActorMessage::GetNextId { responder }) => {
                responder.send(1).unwrap();
            },
            _ => {}
        }
    
    }

    
}

impl Guardian {

    pub fn new(receiver: mpsc::Receiver<Message>) -> Guardian {
        Guardian {
            receiver: receiver,
        }
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            Self::receive(self, message);
        }
    }

    
}

pub struct GetActor {
    pub receiver: mpsc::Receiver<Message> ,
}

impl ActorType for GetActor  {
    

    fn receive(&self, message: Message) {
        match message  {
            Message::GetActorMessage(GetActorMessage::Terminate) => {
                println!("GetActor terminated");
            },
            Message::GetActorMessage(GetActorMessage::GetURI { uri, location }) => {
                println!("URI: {}, Location: {}", uri, location);
            },
            _ => {}
        }

    }

}

impl GetActor {
    pub fn new(receiver: mpsc::Receiver<Message>) -> GetActor {
        GetActor {
            receiver: receiver,
        }
    }
    pub async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message);
        }
    }

}


