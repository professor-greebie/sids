use tokio::sync::mpsc;

use super::messages::{ActorMessage, GetActorMessage};


pub trait ActorType {
    type M;

    fn new(receiver: mpsc::Receiver<Box<Self::M>>) -> Self;
    async fn run(&mut self);
}

pub struct Actor {
    pub receiver: mpsc::Receiver<Box<ActorMessage>> ,
}

impl ActorType for Actor {
    type M = ActorMessage;

    fn new(receiver: mpsc::Receiver<Box<Self::M>>) -> Actor {
        Actor {
            receiver: receiver,
        }
    }

    async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message);
        }
    }
    
}

impl Actor {

    
    
    pub fn receive(&mut self, message: Box<ActorMessage>) {
        match message {
            _ => {}
        }
    }
    
}

pub struct GetActor {
    pub receiver: mpsc::Receiver<Box<GetActorMessage>> ,
}


impl ActorType for GetActor {
    type M = GetActorMessage;
    fn new(receiver: mpsc::Receiver<Box<Self::M>>) -> GetActor {
        GetActor {
            receiver: receiver,
        }
    }
    async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(*message);
        }
    }

}

impl GetActor {
    

    pub fn receive(&mut self, message: GetActorMessage) {
        match message {
            GetActorMessage::Terminate => {}
            GetActorMessage::GetURI { uri, location } => {}
        }

    }

}
