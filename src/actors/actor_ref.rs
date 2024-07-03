use log::{error, info, warn};

use crate::actors::{actor, messages::Message};

use super::messages;

type TokioSender = tokio::sync::mpsc::Sender<messages::Message>;
type StdSender = std::sync::mpsc::Sender<messages::Message>;

pub (super) enum ActorRef {
    TokioActorRef(TokioActorRef),
    BlockingActorRef(BlockingActorRef),
}

#[derive(Clone, Debug)]
pub(super) struct BlockingActorRef {
    sender: StdSender,
}

impl BlockingActorRef {
    pub(super) fn new(actor: actor::Actor, sender: StdSender) -> Self {
        match actor {
            actor::Actor::Collector(collector) => {
                info!(actor = "Collector"; "Spawning collector actor");
                std::thread::spawn(move || {
                    let mut collector = collector;
                    collector.run();
                });
            }
            _ => {
                error!("Actor not found");
            }
        }
        Self { sender }
    }

    pub(super) fn send(&mut self, message: &messages::Message) {
        let (snd, rec) = std::sync::mpsc::channel::<messages::Message>();
        match message {
            messages::Message::CollectorMessage(messages::CollectorMessage::GetURI {
                uri,
                location,
                responder,
            }) => {
                info!(actor = "Collector"; "Getting URI");
                let _ = self.sender.send(Message::CollectorMessage(
                    messages::CollectorMessage::GetURI {
                        uri: uri.to_string(),
                        location: location.to_string(),
                        responder: snd,
                    },
                ));
                let response = rec.recv().unwrap();
                match response {
                    messages::Message::Response(messages::Response::Success) => {
                        info!("Success");
                    }
                    messages::Message::Response(messages::Response::Failure) => {
                        error!("Failure");
                    }
                    _ => {
                        error!("No response received");
                    }
                }
            }
            _ => {
                let _ = self.sender.send(messages::Message::NoMessage);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TokioActorRef {
    sender: tokio::sync::mpsc::Sender<messages::Message>,
}

impl TokioActorRef {
    pub(super) fn new(actor: actor::Actor, snd: TokioSender) -> Self {
        match actor {
            actor::Actor::Guardian(guardian) => {
                info!(actor = "Guardian"; "Spawning guardian actor");
                tokio::spawn(async move {
                    let mut guardian = guardian;
                    guardian.run().await;
                });
            }
            actor::Actor::KafkaProducerActor(kafka_actor) => {
                info!(actor = "Kafka Producer"; "Spawning a Kafka Producing actor");
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

    pub(super) async fn send(&mut self, message: &messages::Message) {
        match message {
            messages::Message::KafkaProducerMessage(messages::KafkaProducerMessage::Produce {
                topic,
                message,
            }) => {
                info!(actor = "Kafka Producer"; "Producing Kafka message");
                let (snd, _rec) = tokio::sync::oneshot::channel();
                let _ = self
                    .sender
                    .send(messages::Message::KafkaProducerMessage(
                        messages::KafkaProducerMessage::ProduceWithResponse {
                            topic: topic.to_string(),
                            message: message.to_string(),
                            responder: snd,
                        },
                    ))
                    .await;
                _rec.await.expect("Actor was killed before send.");
            }
            _ => {
                let _ = self.sender.send(messages::Message::NoMessage).await;
            }
        }
    }
}
