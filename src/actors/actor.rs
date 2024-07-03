use crate::actors::messages;

use kafka::producer::Record;
use log::error;
use log::info;
use log::warn;
use std::io::prelude::*;
use std::io::Error;
use tokio::sync::mpsc;

use super::actor_system::ActorSystem;

pub fn start_actor_system() -> ActorSystem {
    let actor_system = ActorSystem::new();
    actor_system
}

pub fn spawn_actor(actor_system: &mut ActorSystem) -> &ActorSystem {

    // This is a temporary solution to spawn actors. It should be replaced with a more robust solution.
    actor_system
}

pub fn spawn_collector(actor_system: &mut ActorSystem) -> &ActorSystem {
    // This is a temporary solution to spawn actors. It should be replaced with a more robust solution.
    actor_system
}

pub fn spawn_kafka_producer_actor(actor_system:  &mut ActorSystem) -> &ActorSystem {
    // This is a temporary solution to spawn actors. It should be replaced with a more robust solution.
    actor_system
}

pub fn next_actor_system(actor_system: &mut ActorSystem) -> &ActorSystem {
    actor_system
}

pub async fn send_message(actor_system: &ActorSystem, message: messages::Message) {

}

pub fn send_get_request(actor_system: &ActorSystem, uri: String, location: String) {

}

pub fn describe_actor_system(actor_system: &ActorSystem) {
    log::info!(actor = "Actor System"; "hello");

}

// An actor system is a collection of actors that can communicate with each other.
// Begin Linked Node Structure








#[trait_variant::make(HttpService: Send)]

pub(in super) trait ActorType {
    async fn receive(&self, message: messages::Message) -> Result<(), Error>;
}

pub trait SyncActorType {
    fn receive(&self, message: messages::Message) -> Result<(), Error>;
}

pub(super) enum Actor {
    Guardian(Guardian),
    Collector(Collector),
    KafkaProducerActor(KafkaProducerActor),
    NotAnActor,
}


pub(super) struct Guardian {
    receiver: mpsc::Receiver<messages::Message>,
}

impl ActorType for Guardian {
    async fn receive(&self, message: messages::Message) -> Result<(), Error> {
        match message {
            messages::Message::ActorMessage(messages::ActorMessage::Terminate) => {
                println!("Actor terminated");
            }
            messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder }) => {
                responder.send(1).unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

struct LogActor {
    receiver: mpsc::Receiver<messages::Message>,
}

impl ActorType for LogActor {
    async fn receive(&self, message: messages::Message) -> Result<(), Error> {
        match message {
            messages::Message::ActorMessage(messages::ActorMessage::Terminate) => {
                println!("Actor terminated");
            },
            messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder }) => {
                responder.send(1).unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

impl Guardian {
    pub(super) fn new(receiver: mpsc::Receiver<messages::Message>) -> Guardian {
        Guardian { receiver: receiver }
    }

    pub(super) async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

pub(crate) struct Collector {
    receiver: std::sync::mpsc::Receiver<messages::Message>,
}

impl SyncActorType for Collector {
    fn receive(&self, message: messages::Message) -> Result<(), Error> {
        match message {
            messages::Message::CollectorMessage(messages::CollectorMessage::Terminate) => {
                println!("Actor terminated");
            }
            messages::Message::CollectorMessage(messages::CollectorMessage::GetURI {
                uri,
                location,
                responder,
            }) => {
                self.get_uri(uri, location).expect("Failed to get URI");
                responder
                    .send(messages::Message::Response(messages::Response::Success))
                    .unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

impl Collector {
    // Collector requires spawn blocking in order to get the response from the reqwest::blocking::get method.
    pub (super) fn new(receiver: std::sync::mpsc::Receiver<messages::Message>) -> Collector {
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

pub (super) struct KafkaProducerActor {
    receiver: mpsc::Receiver<messages::Message>,
}

impl ActorType for KafkaProducerActor {
    async fn receive(&self, message: messages::Message) -> Result<(), Error> {
        match message {
            messages::Message::KafkaProducerMessage(messages::KafkaProducerMessage::Terminate) => {
                println!("Actor terminated");
            }
            messages::Message::KafkaProducerMessage(messages::KafkaProducerMessage::ProduceWithResponse {
                topic,
                message,
                responder,
            }) => {
                info!("Producing message to topic {}", topic);
                let mut producer =
                    kafka::producer::Producer::from_hosts(vec!["10.172.55.10:29092".to_owned()])
                        .create()
                        .unwrap();
                producer
                    .send(&Record::from_value(topic.as_str(), message.as_bytes()))
                    .unwrap();

                responder
                    .send(messages::Message::Response(messages::Response::Success))
                    .unwrap();
            }

            _ => {}
        }
        Ok(())
    }
}

impl KafkaProducerActor {
    pub (super) fn new(receiver: mpsc::Receiver<messages::Message>) -> KafkaProducerActor {
        KafkaProducerActor { receiver: receiver }
    }

    pub(super) async fn run(&mut self) {
        info!(actor = "Kafka Producer"; "Running Kafka producer actor");
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_actor_system() {
        let mut actor_system = start_actor_system();

    }

}


