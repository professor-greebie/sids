
use kafka::producer::Record;
use log::info;
use tokio::sync::mpsc;
use std::io::prelude::*;
use std::io::Error;



use crate::actors::messages::Response;

use super::messages::KafkaProducerMessage;
use super::messages::{ActorMessage, GetActorMessage, Message};

#[trait_variant::make(HttpService: Send)]

pub trait ActorType {
    async fn receive(&self, message: Message) -> Result<(), Error>;
}

pub trait SyncActorType {
    fn receive(&self, message: Message) -> Result<(), Error>;
}

pub enum Actor {
    Guardian(Guardian),
    GetActor(GetActor),
    KafkaProducerActor(KafkaProducerActor),
    NotAnActor,
}

pub enum SelectActor {
    Guardian,
    GetActor,
    KafkaProducerActor,
}

pub struct Guardian {
    pub receiver: mpsc::Receiver<Message>,
}

impl ActorType for Guardian {
    async fn receive(&self, message: Message) -> Result<(), Error>{
        match message {
            Message::ActorMessage(ActorMessage::Terminate) => {
                println!("Actor terminated");
            }
            Message::ActorMessage(ActorMessage::GetNextId { responder }) => {
                responder.send(1).unwrap();
            }
            _ => {}
        }
        Ok(())
        
    }
}

impl Guardian {
    pub fn new(receiver: mpsc::Receiver<Message>) -> Guardian {
        Guardian { receiver: receiver }
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

pub struct GetActor {
    pub receiver: std::sync::mpsc::Receiver<Message>,
}


impl GetActor {
    // GetActor requires spawn blocking in order to get the response from the reqwest::blocking::get method.
    pub fn new(receiver: std::sync::mpsc::Receiver<Message>) -> GetActor {
        GetActor { receiver: receiver, }
    }

    fn receive(&self, message: Message) -> Result<(), Error>{
        info!("Received message");
        match message {
            Message::GetActorMessage(GetActorMessage::Terminate) =>  {
                println!("GetActor terminated");
            },
            Message::GetActorMessage(GetActorMessage::GetURI { uri, location, responder}) => {
                self.get_uri(uri, location).expect("Failed to get URI");
                responder.send(Message::Response(Response::Success)).unwrap();

            }
            _ => {
                println!("No message received");
            }
        }
        Ok(())
    }
    pub fn run(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            self.receive(message).unwrap();
        }
        
    }

    pub fn get_uri(&self, uri: String, location: String) -> Result<(), Error> {
        info!("Getting URI {}", uri);
        info!("Writing to location {}", location);
        let res = reqwest::blocking::get(uri).unwrap().text().unwrap();
        info!{"Body contains{}", res};
        self.write_to_file(res, location).unwrap();

        Ok(())
    }

    pub fn write_to_file(&self, body: String, location: String) -> Result<(), Error> {
        info!("Writing to file - {}", location);
        let mut file = std::fs::File::create(location).unwrap();
        file.write_all(body.as_bytes()).unwrap();
        Ok(())
    }
}

pub struct KafkaProducerActor {
    receiver: mpsc::Receiver<Message>,
}

impl ActorType for KafkaProducerActor {
    async fn receive(&self, message: Message) -> Result<(), Error>{
        match message {
            Message::KafkaProducerMessage(KafkaProducerMessage::Terminate) => {
                println!("Actor terminated");
            }
            Message::KafkaProducerMessage(KafkaProducerMessage::ProduceWithResponse { topic, message, responder }) => {
                info!("Producing message to topic {}", topic);
                let mut producer = kafka::producer::Producer::from_hosts(vec!("localhost:29092".to_owned())).create().unwrap();
                producer.send(&Record::from_value(topic.as_str(), message.as_bytes())).unwrap();

                responder.send(Message::Response(Response::Success)).unwrap();
            }

            _ => {}
        }
        Ok(())
    }


}

impl KafkaProducerActor {
    pub fn new(receiver: mpsc::Receiver<Message>) -> KafkaProducerActor {
        KafkaProducerActor { receiver: receiver }
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}