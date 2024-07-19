use crate::actors::messages;

use kafka::producer::Record;
use log::info;
use std::io::prelude::*;
use std::io::Error;
use tokio::sync::mpsc;

use super::guardian::Guardian;

#[trait_variant::make(HttpService: Send)]

pub(in super) trait ActorType {
    async fn receive(&self, message: messages::Message) -> Result<(), Error>;
}

pub trait SyncActorType {
    fn receive(&self, message: messages::Message) -> Result<(), Error>;
}

#[allow(dead_code)]
pub(super) enum Actor {
    Guardian(Guardian),
    Collector(Collector),
    CleaningActor(CleaningActor),
    KafkaProducerActor(KafkaProducerActor),
    NotAnActor,
}




struct LogActor {
    _receiver: mpsc::Receiver<messages::Message>,
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



pub(crate) struct Collector {
    receiver: std::sync::mpsc::Receiver<messages::Message>,
}

impl SyncActorType for Collector {
    fn receive(&self, message: messages::Message) -> Result<(), Error> {
        match message {
            messages::Message::CollectorMessage(messages::CollectorMessage::Terminate) => {
                println!("Actor terminated");
            },
            messages::Message::CollectorMessage(messages::CollectorMessage::GetURITemplate { uri, location }) =>
            {
                self.get_uri(uri, location).expect("Failed to get URI");
            },
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

pub(super) struct CleaningActor {
    receiver: mpsc::Receiver<messages::Message>,
}

impl ActorType for CleaningActor {
    async fn receive(&self, message: messages::Message) -> Result<(), Error> {
        match message {
            messages::Message::CleaningActorMessage(messages::CleaningActorMessage::Terminate) => {
                println!("Actor terminated");
            }
            messages::Message::CleaningActorMessage(messages::CleaningActorMessage::Clean { location, responder }) => {
                //self.clean(location).expect("Failed to clean location");
                responder
                    .send(messages::Message::Response(messages::Response::Success))
                    .unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

impl CleaningActor {
    pub(super) fn new(receiver: mpsc::Receiver<messages::Message>) -> CleaningActor {
        CleaningActor { receiver: receiver }
    }

    pub(super) async fn run(&mut self) {
        info!(actor = "Cleaning Actor"; "Running cleaning actor");
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }

    fn clean(&self, location: String) -> Result<(), Error> {
        info!("Cleaning location {}", location);
        std::fs::remove_file(location).unwrap();
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
    async fn test_guardian_actor() {
        let (_tx, rx) = mpsc::channel(1);
        let (tx2, rx2) = oneshot::channel();
        let guardian = Guardian::new(rx);
        let message1 = messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder: tx2 });
        let message2 = messages::Message::ActorMessage(messages::ActorMessage::Terminate);
        guardian.receive(message1).await.unwrap();
        
        let response = rx2.await.unwrap();
        assert_eq!(response, 1);
        guardian.receive(message2).await.expect("Failed to terminate actor");

    }

}


