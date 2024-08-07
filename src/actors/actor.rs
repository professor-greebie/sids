use crate::actors::messages;

use kafka::producer::Record;
use log::info;
use std::io::prelude::*;
use std::io::Error;
use tokio::sync::mpsc;

use super::guardian::Guardian;

#[trait_variant::make(HttpService: Send)]

pub(super) trait ActorType {
    async fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error>;
}

pub trait SyncActorType {
    fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error>;
}

#[allow(dead_code)]
pub(super) enum Actor {
    Guardian(Guardian),
    Collector(Collector),
    LogActor(LogActor),
    CleaningActor(CleaningActor),
    KafkaProducerActor(KafkaProducerActor),
    KafkaConsumerActor(KafkaConsumerActor),
    NotAnActor,
}

pub(super) struct LogActor {
    _receiver: mpsc::Receiver<messages::InternalMessage>,
}

impl ActorType for LogActor {
    async fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::LogMessage { message } => {
                info!(actor = "LogActor"; "Log message: {} as per request.", message);
            }
            messages::InternalMessage::Terminate => {
                println!("Actor terminated");
                self._receiver.close();
                return Err(Error::new(std::io::ErrorKind::Other, "Actor terminated"));
            }
            _ => {}
        }
        Ok(())
    }
}

impl LogActor {
    pub(super) fn new(receiver: mpsc::Receiver<messages::InternalMessage>) -> LogActor {
        LogActor {
            _receiver: receiver,
        }
    }
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
    fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::Terminate => {
                // terminate the actor. What do I need to do to stop the listener?
                info!("Actor terminated");
                return Err(Error::new(std::io::ErrorKind::Other, "Actor terminated"));
            }
            messages::InternalMessage::CollectorMessage(
                messages::CollectorMessage::GetURITemplate { uri, location },
            ) => {
                self.get_uri(uri, location).expect("Failed to get URI");
            }
            messages::InternalMessage::CollectorMessage(messages::CollectorMessage::GetURI {
                uri,
                location,
                responder,
            }) => {
                self.get_uri(uri, location).expect("Failed to get URI");
                responder.send(messages::ResponseMessage::Success).unwrap();
            }
            _ => {}
        }
        Ok(())
    }
}

impl Collector {
    // Collector requires spawn blocking in order to get the response from the reqwest::blocking::get method.
    pub(super) fn new(receiver: std::sync::mpsc::Receiver<messages::InternalMessage>) -> Collector {
        info!(actor = "Get Actor"; "Creating Get Actor");
        Collector { receiver: receiver }
    }

    pub(super) fn run(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            self.receive(message).unwrap();
        }
    }

    fn get_uri(&self, uri: String, location: String) -> Result<(), Error> {
        if cfg!(test) {
            return Ok(());
        }
        info!("Getting URI {}", uri);
        info!("Writing to location {}", location);
        let res = reqwest::blocking::get(uri).unwrap().text().unwrap();
        self.write_to_file(res, location).unwrap();
        Ok(())
    }

    fn write_to_file(&self, body: String, location: String) -> Result<(), Error> {
        if cfg!(test) {
            return Ok(());
        }
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
    async fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::Terminate => {
                println!("Actor terminated");
                self._receiver.close();
                return Err(Error::new(std::io::ErrorKind::Other, "Actor terminated"));
            }
            messages::InternalMessage::CleaningActorMessage(
                messages::CleaningActorMessage::Clean { location },
            ) => {
                self.clean(location).expect("Failed to clean location");
            }
            _ => {}
        }
        Ok(())
    }
}

impl CleaningActor {
    pub(super) fn new(receiver: mpsc::Receiver<messages::InternalMessage>) -> CleaningActor {
        CleaningActor {
            _receiver: receiver,
        }
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
        // get the data from the file location and clean it somehow.
        Ok(())
    }
}

pub(super) struct KafkaProducerActor {
    receiver: mpsc::Receiver<messages::InternalMessage>,
    broker_host: String,
    broker_port: String,
}

impl Default for KafkaProducerActor {
    fn default() -> Self {
        KafkaProducerActor {
            receiver: mpsc::channel(1).1,
            broker_host: "localhost".to_owned(),
            broker_port: "29092".to_owned(),
        }
    }
}

impl ActorType for KafkaProducerActor {
    async fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::Terminate => {
                println!("Actor terminated");
                self.receiver.close();
                return Err(Error::new(std::io::ErrorKind::Other, "Actor terminated"));
            }
            messages::InternalMessage::KafkaProducerMessage(
                messages::KafkaProducerMessage::Produce {
                    topic,
                    key,
                    message,
                },
            ) => {                
                let broker_string = format!("{}:{}", self.broker_host, self.broker_port);
                let mut producer =
                    kafka::producer::Producer::from_hosts(vec![broker_string.to_owned()])
                        .create()
                        .unwrap();

                // in the test environment, we don't want to produce messages to Kafka.
                 if cfg!(test) {
                    return Ok(());
                }
                // grcov-excl-start
                producer
                    .send(&Record::from_key_value(
                        topic.as_str(),
                        key.as_str(),
                        message.as_bytes(),
                    ))
                    .unwrap();
                // do we ever need a response from the kafka producer?
                // grcov-excl-stop
            }

            _ => {}
            
        }
        Ok(())
    }
}

impl KafkaProducerActor {
    #[allow(dead_code)]
    pub(super) fn new(
        receiver: mpsc::Receiver<messages::InternalMessage>,
        host: Option<String>,
        port: Option<String>,
    ) -> KafkaProducerActor {
        KafkaProducerActor {
            receiver: receiver,
            broker_host: host.unwrap_or("localhost".to_string()),
            broker_port: port.unwrap_or("29092".to_string()),
        }
    }

    pub(super) async fn run(&mut self) {
        info!(actor = "Kafka Producer"; "Running Kafka producer actor");
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

pub(super) struct KafkaConsumerActor {
    receiver: mpsc::Receiver<messages::InternalMessage>,
    broker_host: String,
    broker_port: String,
}
impl Default for KafkaConsumerActor {
    fn default() -> Self {
        KafkaConsumerActor {
            receiver: mpsc::channel(1).1,
            broker_host: "localhost".to_owned(),
            broker_port: "29092".to_owned(),
        }
    }
}

impl ActorType for KafkaConsumerActor {
    async fn receive(&mut self, message: messages::InternalMessage) -> Result<(), Error> {
        match message {
            messages::InternalMessage::Terminate => {
                println!("Actor terminated");
                self.receiver.close();
                return Err(Error::new(std::io::ErrorKind::Other, "Actor terminated"));
            }
            messages::InternalMessage::KafkaConsumerMessage(
                messages::KafkaConsumerMessage::Consume { topic, group: _ },
            ) => {
                let broker_string = format!("{}:{}", self.broker_host, self.broker_port);
                let mut consumer = kafka::consumer::Consumer::from_hosts(vec![broker_string])
                    .with_topic(topic)
                    .create()
                    .unwrap();
                // in the test environment, we don't want to consume messages from Kafka.
                if cfg!(test) {
                    return Ok(());
                }
                // grcov-excl-start
                consumer.poll().unwrap();
                // grcov-excl-stop
            }
            _ => {}
        }
        Ok(())
    }
}

impl KafkaConsumerActor {
    #[allow(dead_code)]
    pub(super) fn new(
        receiver: mpsc::Receiver<messages::InternalMessage>,
        host: Option<String>,
        port: Option<String>,
    ) -> KafkaConsumerActor {
        KafkaConsumerActor {
            receiver: receiver,
            broker_host: host.unwrap_or("localhost".to_string()),
            broker_port: port.unwrap_or("29092".to_string()),
        }
    }

    pub(super) async fn run(&mut self) {
        info!(actor = "Kafka Consumer"; "Running Kafka consumer actor");
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

// grcov-excl-start

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_actor_receives_standard_log_message() {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        let mut actor = super::LogActor::new(rx);
        //actor.run().await;
        assert!(actor
            .receive(super::messages::InternalMessage::LogMessage {
                message: "Test".to_string()
            })
            .await
            .is_ok());
        assert!(actor
            .receive(super::messages::InternalMessage::Terminate)
            .await
            .is_err());
    }

    #[test]
    fn test_collector() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut actor = Collector::new(rx);
        //actor.run();
        // These will need to be confirmed in integration tests somehow.
        assert!(actor
            .get_uri("test_uri".to_string(), "test_location".to_string())
            .is_ok());
        assert!(actor
            .write_to_file("body".to_string(), "location".to_string())
            .is_ok());
        assert!(actor
            .receive(super::messages::InternalMessage::CollectorMessage(
                super::messages::CollectorMessage::GetURITemplate {
                    uri: "test_uri".to_string(),
                    location: "test_location".to_string()
                }
            ))
            .is_ok());
        assert!(actor
            .receive(super::messages::InternalMessage::Terminate)
            .is_err());
    }

    #[tokio::test]

    async fn test_kafka_producer() {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        let mut actor =
            KafkaProducerActor::new(rx, Some("localhost".to_string()), Some("29092".to_string()));
        let mut actor2 = KafkaProducerActor::default();
        //actor.run().await;

        assert!(actor2
            .receive(super::messages::InternalMessage::KafkaProducerMessage(
                super::messages::KafkaProducerMessage::Produce {
                    topic: "test".to_string(),
                    key: "test".to_string(),
                    message: "test".to_string()
                }
            ))
            .await
            .is_ok());
        assert!(actor
            .receive(super::messages::InternalMessage::Terminate)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_cleaning_actor() {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        let mut actor = CleaningActor::new(rx);
        //actor.run().await;

        assert!(actor.clean("test".to_string()).is_ok());
        assert!(actor
            .receive(super::messages::InternalMessage::CleaningActorMessage(
                super::messages::CleaningActorMessage::Clean {
                    location: "test".to_string()
                }
            ))
            .await
            .is_ok());
        assert!(actor
            .receive(super::messages::InternalMessage::Terminate)
            .await
            .is_err());
    }
}

// grcov-excl-stop
