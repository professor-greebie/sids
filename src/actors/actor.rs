use kafka::producer::Record;
use log::error;
use log::info;
use log::warn;
use std::io::prelude::*;
use std::io::Error;
use tokio::sync::mpsc;

pub fn start_actor_system() -> ActorSystem {
    let mut _actor_system = ActorSystem::new();
    _actor_system
}

pub fn spawn_actor(actor_system: &mut ActorSystem) -> &ActorSystem {
    actor_system.spawn_actor(SelectActor::LogActor);
    actor_system
}

pub fn spawn_get_actor(actor_system: &mut ActorSystem) -> &ActorSystem {
    actor_system.spawn_actor(SelectActor::GetActor);
    actor_system
}

pub fn spawn_kafka_producer_actor(actor_system: &mut ActorSystem) -> &mut ActorSystem {
    actor_system.spawn_actor(SelectActor::KafkaProducerActor);
    actor_system
}

pub fn next_actor_system(actor_system: ActorSystem) -> ActorSystem {
    let system = actor_system.next_actor();
    system
}

pub fn send_message(actor_system: ActorSystem, message: Message) {
    actor_system.send_message(message);
}

pub fn send_get_request(actor_system: &ActorSystem, uri: String, location: String) {
    actor_system.send_get_request(uri, location);
}

pub fn describe_actor_system(actor_system: &ActorSystem) {
    log::info!("Actor system: {:?}", "hello");
}

// An actor system is a collection of actors that can communicate with each other.
#[derive(Clone)]
pub struct ActorSystem {
    // The current actor reference.
    _value: Option<ActorRef>,
    _actors: Option<Box<ActorSystem>>,
}

impl ActorSystem {
    fn new() -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(4);
        let sender = SenderType::TokioSender(snd);
        let actor = Actor::Guardian(Guardian::new(rec));
        let actor_ref = ActorRef::new(actor, sender);
        ActorSystem {
            _value: Some(actor_ref),
            _actors: None,
        }
    }

    async fn send_message(&self, message: Message) -> Result<(), Error> {
        match self._value.clone() {
            Some(mut refer) => {
                refer.send(message).await;
                Ok(())
            }
            None => {
                error!("No actor reference found");
                Err(Error::new(std::io::ErrorKind::Other, "No actor reference found"))
        }
    }}

    fn send_get_request(&self, uri: String, location: String) {
        match self._value.clone() {
            Some(refer) => {
                refer.send_get_request(uri, location);
            }
            None => {
                error!("No actor reference found");
            }
        }
}

    // Send a message to the actor system.

    fn spawn_actor(&mut self, actor_select: SelectActor) {
        match actor_select {
            SelectActor::GetActor => {
                let (snd, rec) = std::sync::mpsc::channel::<Message>();
                let sender = SenderType::StdSender(snd);
                let get_actor = GetActor::new(rec);
                let actor = Actor::GetActor(get_actor);
                let actor_ref = ActorRef::new(actor, sender);
                self._actors = Some(Box::new(ActorSystem {
                    _value: Some(actor_ref),
                    _actors: None,
                }));
            }
            SelectActor::KafkaProducerActor => {
                let (snd, rec) = tokio::sync::mpsc::channel::<Message>(32);
                let sender = SenderType::TokioSender(snd);
                let actor = Actor::KafkaProducerActor(KafkaProducerActor::new(rec));
                let actor_ref = ActorRef::new(actor, sender);
                self._actors = Some(Box::new(ActorSystem {
                    _value: Some(actor_ref),
                    _actors: None,
                }));
            }
            _ => {}
        }
    }

    fn next_actor(&self) -> ActorSystem {
        *self._actors.clone().unwrap()
    }
}

#[derive(Clone)]
pub enum SenderType {
    TokioSender(tokio::sync::mpsc::Sender<Message>),
    StdSender(std::sync::mpsc::Sender<Message>),
}

#[derive(Clone)]

struct ActorRef {
    sender: SenderType,
}

impl ActorRef {
    fn new(actor: Actor, snd: SenderType) -> Self {
        match actor {
            Actor::Guardian(guardian) => {
                info!("Spawning guardian actor");
                tokio::spawn(async move {
                    let mut guardian = guardian;
                    guardian.run().await;
                });
            }
            Actor::GetActor(get_actor) => {
                info!("Spawning get actor with std sender");
                std::thread::spawn(move || {
                    let mut get_actor = get_actor;
                    get_actor.run();
                });
            }
            Actor::KafkaProducerActor(kafka_actor) => {
                info!("Spawning a Kafka Producing actor");
                tokio::spawn(async move {
                    let mut kafka_actor = kafka_actor;
                    kafka_actor.run().await;
                });
            }
            _ => {
                error!("Actor not found");
            }
        }
        Self { sender: snd }
    }

    async fn send(&mut self, message: Message) {
        match &self.sender {
            SenderType::TokioSender(sender) => match message {
                Message::KafkaProducerMessage(KafkaProducerMessage::Produce { topic, message }) => {
                    info!("Producing Kafka message");
                    let (snd, _rec) = tokio::sync::oneshot::channel();
                    let _ = sender
                        .send(Message::KafkaProducerMessage(
                            KafkaProducerMessage::ProduceWithResponse {
                                topic,
                                message,
                                responder: snd,
                            },
                        ))
                        .await;
                    _rec.await.expect("Actor was killed before send.");
                }
                _ => {
                    let _ = sender.send(message).await;
                }
            },
            SenderType::StdSender(_sender) => {
                warn!("Std sender should not be sent via async implementation");
            }
        };
    }

    fn send_get_request(self, uri: String, location: String) -> () {
        let (snd, rec) = std::sync::mpsc::channel::<Message>();
        match self.sender {
            SenderType::TokioSender(_sender) => {
                warn!("Tokio sender should not be sent via sync implementation")
            }
            SenderType::StdSender(sender) => {
                let _ = sender
                    .send(Message::GetActorMessage(GetActorMessage::GetURI {
                        uri,
                        location,
                        responder: snd,
                    }))
                    .unwrap();
            }
        }
        let response = rec.recv().unwrap();
        match response {
            Message::Response(Response::Success) => {
                info!("Success");
            }
            Message::Response(Response::Failure) => {
                error!("Failure");
            }
            _ => {
                error!("No response received");
            }
        }
    }
}

#[derive(Debug)]
pub enum Message {
    ActorMessage(ActorMessage),
    GetActorMessage(GetActorMessage),
    KafkaProducerMessage(KafkaProducerMessage),
    Response(Response),
}

#[derive(Debug)]
pub enum ActorMessage {
    Terminate,
    GetNextId {
        responder: tokio::sync::oneshot::Sender<u64>,
    },
}

#[derive(Debug)]

pub enum Response {
    Success,
    Failure,
}

#[derive(Debug)]

pub enum GetActorMessage {
    Terminate,
    GetURI {
        uri: String,
        location: String,
        responder: std::sync::mpsc::Sender<Message>,
    },
}

#[derive(Debug)]

pub enum KafkaProducerMessage {
    Terminate,
    Produce {
        topic: String,
        message: String,
    },
    ProduceWithResponse {
        topic: String,
        message: String,
        responder: tokio::sync::oneshot::Sender<Message>,
    },
}

#[trait_variant::make(HttpService: Send)]

trait ActorType {
    async fn receive(&self, message: Message) -> Result<(), Error>;
}

trait SyncActorType {
    fn receive(&self, message: Message) -> Result<(), Error>;
}

enum Actor {
    Guardian(Guardian),
    GetActor(GetActor),
    KafkaProducerActor(KafkaProducerActor),
    NotAnActor,
}

enum SelectActor {
    Guardian,
    LogActor,
    GetActor,
    KafkaProducerActor,
}

struct Guardian {
    receiver: mpsc::Receiver<Message>,
}

impl ActorType for Guardian {
    async fn receive(&self, message: Message) -> Result<(), Error> {
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

struct LogActor {
    receiver: mpsc::Receiver<Message>,
}

impl ActorType for LogActor {
    async fn receive(&self, message: Message) -> Result<(), Error> {
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
    fn new(receiver: mpsc::Receiver<Message>) -> Guardian {
        Guardian { receiver: receiver }
    }

    async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

struct GetActor {
    receiver: std::sync::mpsc::Receiver<Message>,
}

impl GetActor {
    // GetActor requires spawn blocking in order to get the response from the reqwest::blocking::get method.
    fn new(receiver: std::sync::mpsc::Receiver<Message>) -> GetActor {
        GetActor { receiver: receiver }
    }

    fn receive(&self, message: Message) -> Result<(), Error> {
        info!("Received message");
        match message {
            Message::GetActorMessage(GetActorMessage::Terminate) => {
                println!("GetActor terminated");
            }
            Message::GetActorMessage(GetActorMessage::GetURI {
                uri,
                location,
                responder,
            }) => {
                self.get_uri(uri, location).expect("Failed to get URI");
                responder
                    .send(Message::Response(Response::Success))
                    .unwrap();
            }
            _ => {
                println!("No message received");
            }
        }
        Ok(())
    }
    fn run(&mut self) {
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

struct KafkaProducerActor {
    receiver: mpsc::Receiver<Message>,
}

impl ActorType for KafkaProducerActor {
    async fn receive(&self, message: Message) -> Result<(), Error> {
        match message {
            Message::KafkaProducerMessage(KafkaProducerMessage::Terminate) => {
                println!("Actor terminated");
            }
            Message::KafkaProducerMessage(KafkaProducerMessage::ProduceWithResponse {
                topic,
                message,
                responder,
            }) => {
                info!("Producing message to topic {}", topic);
                let mut producer =
                    kafka::producer::Producer::from_hosts(vec!["localhost:29092".to_owned()])
                        .create()
                        .unwrap();
                producer
                    .send(&Record::from_value(topic.as_str(), message.as_bytes()))
                    .unwrap();

                responder
                    .send(Message::Response(Response::Success))
                    .unwrap();
            }

            _ => {}
        }
        Ok(())
    }
}

impl KafkaProducerActor {
    fn new(receiver: mpsc::Receiver<Message>) -> KafkaProducerActor {
        KafkaProducerActor { receiver: receiver }
    }

    async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}
