use crate::actors::messages;

use kafka::producer::Record;
use log::info;
use std::io::prelude::*;
use std::io::Error;
use tokio::sync::mpsc;

use super::guardian::Guardian;

#[trait_variant::make(HttpService: Send)]

pub(in super) trait ActorType {
    async fn receive(&self, message: messages::InternalMessage) -> Result<(), Error>;
}

pub trait SyncActorType {
    fn receive(&self, message: messages::InternalMessage) -> Result<(), Error>;
}

#[allow(dead_code)]
pub(super) enum Actor {
    Guardian(Guardian),
    Collector(Collector),
    LogActor(LogActor),
    CleaningActor(CleaningActor),
    KafkaProducerActor(KafkaProducerActor),
    NotAnActor,
}




pub (super) struct LogActor {
    _receiver: mpsc::Receiver<messages::InternalMessage>,
}

impl ActorType for LogActor {
    async fn receive(&self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::ActorMessage(messages::ActorMessage::Terminate) => {
                println!("Actor terminated");
            },
            messages::InternalMessage::ActorMessage(messages::ActorMessage::GetNextId) => {
                //responder.send(1).unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

impl LogActor {
    #[allow(dead_code)]
    pub(super) fn new(receiver: mpsc::Receiver<messages::InternalMessage>) -> LogActor {
        LogActor { _receiver: receiver }
    }

    #[allow(dead_code)]
    pub(super) async fn run(&mut self) {
        info!(actor = "Log Actor"; "Running log actor");
        while let Some(message) = self._receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }

}



pub(crate) struct Collector {
    receiver: std::sync::mpsc::Receiver<messages::InternalMessage>,
}

impl SyncActorType for Collector {
    fn receive(&self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::CollectorMessage(messages::CollectorMessage::Terminate) => {
                println!("Actor terminated");
            },
            messages::InternalMessage::CollectorMessage(messages::CollectorMessage::GetURITemplate { uri, location }) =>
            {
                self.get_uri(uri, location).expect("Failed to get URI");
            },
            messages::InternalMessage::CollectorMessage(messages::CollectorMessage::GetURI {
                uri,
                location,
                responder,
            }) => {
                self.get_uri(uri, location).expect("Failed to get URI");
                responder
                    .send(messages::ResponseMessage::Response(messages::Response::Success))
                    .unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

impl Collector {
    // Collector requires spawn blocking in order to get the response from the reqwest::blocking::get method.
    pub (super) fn new(receiver: std::sync::mpsc::Receiver<messages::InternalMessage>) -> Collector {
        info!(actor = "Get Actor"; "Creating Get Actor");
        Collector { receiver: receiver }
    }

    pub(super) fn run(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            self.receive(message).unwrap();
        }
    }

    fn get_uri(&self, uri: String, location: String) -> Result<(), Error> {
        info!("Getting URI {}", uri);
        info!("Writing to location {}", location);
        let res = reqwest::blocking::get(uri).unwrap().text().unwrap();
        info! {"Body contains{}", res};
        self.write_to_file(res, location).unwrap();

        Ok(())
    }

    fn write_to_file(&self, body: String, location: String) -> Result<(), Error> {
        info!("Writing to file - {}", location);
        let mut file = std::fs::File::create(location).unwrap();
        file.write_all(body.as_bytes()).unwrap();
        Ok(())
    }
}

pub(super) struct CleaningActor {

    _receiver: mpsc::Receiver<messages::InternalMessage>,
}

impl ActorType for CleaningActor {
    async fn receive(&self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::CleaningActorMessage(messages::CleaningActorMessage::Terminate) => {
                println!("Actor terminated");
            }
            messages::InternalMessage::CleaningActorMessage(messages::CleaningActorMessage::Clean { location: _ }) => {
                //self.clean(location).expect("Failed to clean location");
            }
            _ => {}
        }
        Ok(())
    }
}

impl CleaningActor {
    pub(super) fn new(receiver: mpsc::Receiver<messages::InternalMessage>) -> CleaningActor {
        CleaningActor { _receiver: receiver }
    }

    #[allow(dead_code)]
    pub(super) async fn run(&mut self) {
        info!(actor = "Cleaning Actor"; "Running cleaning actor");
        while let Some(message) = self._receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }

    #[allow(dead_code)]
    fn clean(&self, location: String) -> Result<(), Error> {
        info!("Cleaning location {}", location);
        std::fs::remove_file(location).unwrap();
        Ok(())
    }
}




pub (super) struct KafkaProducerActor {
    receiver: mpsc::Receiver<messages::InternalMessage>,
    broker_host: String,
    broker_port: String,
}

impl Default for KafkaProducerActor {
    fn default() -> Self {
        KafkaProducerActor {
            receiver: mpsc::channel(1).1,
            broker_host: "localhost".to_owned(),
            broker_port: "29092".to_owned(),}
        }
    }

impl ActorType for KafkaProducerActor {
    async fn receive(&self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::KafkaProducerMessage(messages::KafkaProducerMessage::Terminate) => {
                println!("Actor terminated");
            }
            messages::InternalMessage::KafkaProducerMessage(messages::KafkaProducerMessage::Produce {
                topic,
                key,
                message,
                responder: _, 
            }) => {
                info!("Producing message to topic {}", topic);
                let broker_string = format!("{}:{}", self.broker_host, self.broker_port);
                let mut producer =
                    kafka::producer::Producer::from_hosts(vec![broker_string.to_owned()])
                        .create()
                        .unwrap();
                producer
                    .send(&Record::from_key_value(topic.as_str(), key.as_str(), message.as_bytes()))
                    .unwrap();
            }

            _ => {}
        }
        Ok(())
    }
}

impl KafkaProducerActor {
    #[allow(dead_code)]
    pub (super) fn new(receiver: mpsc::Receiver<messages::InternalMessage>, host: Option<String>, port: Option<String>) -> KafkaProducerActor {
        KafkaProducerActor { receiver: receiver, broker_host: host.unwrap_or("localhost".to_string()), broker_port: port.unwrap_or("29092".to_string()) }
    }

    pub(super) async fn run(&mut self) {
        info!(actor = "Kafka Producer"; "Running Kafka producer actor");
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}



