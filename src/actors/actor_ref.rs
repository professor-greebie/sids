use std::io::Error;

use log::{error, info};

use crate::actors::{actor, guardian::Guardian, messages::InternalMessage};

use super::messages;

type TokioSender = tokio::sync::mpsc::Sender<messages::InternalMessage>;
type StdSender = std::sync::mpsc::Sender<messages::InternalMessage>;
type GuardianSender = tokio::sync::mpsc::Sender<messages::GuardianMessage>;

#[allow(dead_code)]
#[derive(Debug)]
pub(super) enum ActorRef {
    TokioActorRef(TokioActorRef),
    BlockingActorRef(BlockingActorRef),
}

#[derive(Clone, Debug)]
pub(super) struct GuardianActorRef {
    sender: GuardianSender,
}

impl GuardianActorRef {
    pub(super) fn new(guardian: Guardian, snd: GuardianSender) -> Self {
        info!(actor = "Guardian"; "Spawning guardian actor");
        tokio::spawn(async move {
            let mut guardian = guardian;
            guardian.run().await;
        });
        Self { sender: snd }
    }

    pub(super) async fn send(&mut self, message: messages::GuardianMessage) {
        match message {
            messages::GuardianMessage::Terminate => {
                info!(actor = "Guardian"; "Terminating guardian actor");
                let _ = self.sender.send(messages::GuardianMessage::Terminate).await;
            }
            messages::GuardianMessage::Dispatch {
                officer_id,
                message,
            } => {
                info!(actor = "Guardian"; "Dispatching message to officer {}", officer_id);
                let _ = self
                    .sender
                    .send(messages::GuardianMessage::Dispatch {
                        officer_id: officer_id,
                        message: message.clone(),
                    })
                    .await;
            }
            messages::GuardianMessage::CreateOfficer {
                officer_type,
                responder,
            } => {
                info!(actor = "Guardian"; "Creating officer");
                let (tx, rx) = tokio::sync::oneshot::channel::<messages::ResponseMessage>();
                let _ = self
                    .sender
                    .send(messages::GuardianMessage::CreateOfficer {
                        officer_type: officer_type,
                        responder: tx,
                    })
                    .await;
                let response = match rx.await {
                    Ok(messages::ResponseMessage::Success) => {
                        info!("Success");
                        messages::ResponseMessage::Success
                    }
                    Ok(messages::ResponseMessage::Failure) => {
                        error!("Failure");
                        messages::ResponseMessage::Failure
                    }
                    _ => {
                        error!("No response received");
                        messages::ResponseMessage::Failure
                    }
                };

                responder.send(response).unwrap();
            }
            messages::GuardianMessage::RemoveOfficer {
                officer_id,
                responder,
            } => {
                info!(actor = "Guardian"; "Removing officer with id: {}", officer_id);
                let (tx, rx) = tokio::sync::oneshot::channel::<messages::ResponseMessage>();
                let _ = self
                    .sender
                    .send(messages::GuardianMessage::RemoveOfficer {
                        officer_id: officer_id,
                        responder: tx,
                    })
                    .await;
                responder.send(rx.await.unwrap()).unwrap();
            }
            messages::GuardianMessage::AddCourrier {
                officer_id,
                courrier_type,
                responder,
            } => {
                info!(actor = "Guardian"; "Adding courrier to officer with id: {}", officer_id);
                let (tx, rx) = tokio::sync::oneshot::channel::<messages::ResponseMessage>();
                let _ = self
                    .sender
                    .send(messages::GuardianMessage::AddCourrier {
                        officer_id: officer_id,
                        courrier_type: courrier_type,
                        responder: tx,
                    })
                    .await;
                responder.send(rx.await.unwrap()).unwrap();
            }
            messages::GuardianMessage::RemoveCourrier {
                officer_id,
                courrier_id,
                responder,
            } => {
                info!(actor = "Guardian"; "Removing courrier with id: {} from officer with id: {}", courrier_id, officer_id);
                let (tx, rx) = tokio::sync::oneshot::channel::<messages::ResponseMessage>();
                let _ = self
                    .sender
                    .send(messages::GuardianMessage::RemoveCourrier {
                        officer_id,
                        courrier_id,
                        responder: tx,
                    })
                    .await;
                responder.send(rx.await.unwrap()).unwrap();
            }
            messages::GuardianMessage::NoMessage => {
                info!(actor = "Guardian"; "No message (ping) sent to guardian actor.");
                let _ = self.sender.send(messages::GuardianMessage::NoMessage).await;
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(super) struct BlockingActorRef {
    sender: StdSender,
}

impl BlockingActorRef {
    pub(super) fn new(actor: actor::Actor, sender: StdSender) -> Result<Self, Error> {
        match actor {
            actor::Actor::Collector(collector) => {
                info!(actor = "Collector"; "Spawning collector actor.");
                std::thread::spawn(move || {
                    let mut collector = collector;
                    collector.run();
                });
            }
            _ => {
                error!("Actor not found.");
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Actor not found.",
                ));
            }
        }

        Ok(Self { sender })
    }

    pub(super) fn send(&mut self, message: &messages::InternalMessage) {
        let (snd, rec) = std::sync::mpsc::channel();
        match message {
            messages::InternalMessage::CollectorMessage(
                messages::CollectorMessage::GetURITemplate { uri, location },
            ) => {
                info!(actor = "Collector"; "Getting URI");
                let _ = self.sender.send(InternalMessage::CollectorMessage(
                    messages::CollectorMessage::GetURI {
                        uri: uri.to_string(),
                        location: location.to_string(),
                        responder: snd,
                    },
                ));
                let response = rec.recv().unwrap();
                match response {
                    messages::ResponseMessage::Success => {
                        info!("Success");
                    }
                    messages::ResponseMessage::Failure => {
                        error!("Failure");
                    }
                    _ => {
                        error!("No response received");
                    }
                }
            }
            _ => {
                let _ = self.sender.send(messages::InternalMessage::NoMessage);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TokioActorRef {
    sender: TokioSender,
}

impl TokioActorRef {
    pub(super) fn new(actor: actor::Actor, snd: TokioSender) -> Result<Self, Error> {
        match actor {
            actor::Actor::Guardian(_guardian) => {
                error!("Guardian actor should not be spawned as a regular Tokio actor.");
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Guardian actor should not be spawned as a regular Tokio actor.",
                ));
            }
            actor::Actor::Collector(_collector) => {
                error!("Collector actor should not be spawned as a regular Tokio actor. Use BlockingActorRef instead.");
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "Collector actor should not be spawned as a regular Tokio actor. Use BlockingActorRef instead."));
            }
            actor::Actor::LogActor(log_actor) => {
                info!(actor = "Log Actor"; "Spawning a Log actor");
                tokio::spawn(async move {
                    let mut log_actor = log_actor;
                    log_actor.run().await;
                });
            }
            actor::Actor::CleaningActor(cleaning_actor) => {
                info!(actor = "Cleaning Actor"; "Spawning a Cleaning actor");
                tokio::spawn(async move {
                    let mut cleaning_actor = cleaning_actor;
                    cleaning_actor.run().await;
                });
            }
            actor::Actor::KafkaProducerActor(kafka_actor) => {
                info!(actor = "Kafka Producer"; "Spawning a Kafka Producing actor");
                tokio::spawn(async move {
                    let mut kafka_actor = kafka_actor;
                    kafka_actor.run().await;
                });
            }
            actor::Actor::KafkaConsumerActor(kafka_actor) => {
                info!(actor = "Kafka Consumer"; "Spawning a Kafka Consuming actor");
                tokio::spawn(async move {
                    let mut kafka_actor = kafka_actor;
                    kafka_actor.run().await;
                });
            }
            _ => {
                error!("Actor not found");
                return Err(Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Actor not found.",
                ));
            }
        }
        Ok(Self { sender: snd })
    }

    pub(super) async fn send(&mut self, message: &messages::InternalMessage) {
        // Need a way to reduce the complexity of InternalMessages vis a vis the actors in the system.
        match message {
            messages::InternalMessage::KafkaProducerMessage(
                messages::KafkaProducerMessage::Produce {
                    topic,
                    key,
                    message,
                },
            ) => {
                info!(actor = "Kafka Producer"; "Producing Kafka message");
                let _ = self
                    .sender
                    .send(messages::InternalMessage::KafkaProducerMessage(
                        messages::KafkaProducerMessage::Produce {
                            topic: topic.to_string(),
                            key: key.to_string(),
                            message: message.to_string(),
                        },
                    ))
                    .await;
            }
            messages::InternalMessage::LogMessage { message } => {
                // send the responder or force the actor reference to send the response?
                let msg = messages::InternalMessage::LogMessage {
                    message: message.to_string(),
                };
                let _ = self.sender.send(msg).await;
                // do something with the id;
                // do we ever need a response from the logger?
            }

            messages::InternalMessage::Terminate => {
                let _ = self.sender.send(messages::InternalMessage::Terminate).await;
            }
            _ => {
                let _ = self.sender.send(messages::InternalMessage::NoMessage).await;
            }
        }
    }
}

// grcov-excl-start
#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use actor::LogActor;

    use super::*;

    #[tokio::test]
    async fn test_guardian_actor_ref() {
        let (snd, rec) = tokio::sync::mpsc::channel::<messages::GuardianMessage>(10);
        let actor = Guardian::new(rec);
        let mut actor_ref = GuardianActorRef::new(actor, snd);
        let message0 = messages::GuardianMessage::NoMessage;
        let message1 = messages::GuardianMessage::Dispatch {
            officer_id: 1,
            message: messages::Message::LogMessage {
                message: "Hello, world".to_string(),
            },
        };
        let message2 = messages::GuardianMessage::Terminate;
        actor_ref.send(message0).await;
        actor_ref.send(message1).await;
        actor_ref.send(message2).await;
        assert!(true);
    }

    #[tokio::test]
    async fn test_tokio_actor_ref() {
        let (snd, rec) = tokio::sync::mpsc::channel::<InternalMessage>(10);
        let (_snd2, _rec2) = tokio::sync::oneshot::channel::<u64>();
        let (snd_kafka, rec_kafka) = tokio::sync::mpsc::channel::<InternalMessage>(10);
        let (snd_nne, _rec_nne) = tokio::sync::mpsc::channel::<InternalMessage>(5);
        let kafka =
            actor::Actor::KafkaProducerActor(actor::KafkaProducerActor::new(rec_kafka, None, None));
        let actor = actor::Actor::LogActor(LogActor::new(rec));
        let no_actor = actor::Actor::NotAnActor;
        let mut actor_ref = TokioActorRef::new(actor, snd).unwrap();
        let mut kafka_ref = TokioActorRef::new(kafka, snd_kafka).unwrap();
        let actor_ref_no = TokioActorRef::new(no_actor, snd_nne).err().unwrap();
        assert!(matches!(actor_ref_no.kind(), ErrorKind::InvalidInput));

        // the message never uses the responder here. Should we change the message to not include the responder?
        let message = messages::InternalMessage::LogMessage {
            message: "Hello, world".to_string(),
        };
        let _ = actor_ref.send(&message).await;
        let no_message = messages::InternalMessage::NoMessage;
        let kafka_message = messages::InternalMessage::KafkaProducerMessage(
            messages::KafkaProducerMessage::Produce {
                topic: "test".to_string(),
                key: "key".to_string(),
                message: "message".to_string(),
            },
        );
        let _ = actor_ref.send(&no_message).await;
        let _ = kafka_ref.send(&kafka_message).await;
        assert!(true);
    }

    #[test]
    fn test_blocking_actor_ref() {
        let (snd, rec) = std::sync::mpsc::channel::<InternalMessage>();
        let actor = actor::Actor::Collector(actor::Collector::new(rec));
        let mut actor_ref = BlockingActorRef::new(actor, snd).unwrap();
        let message = messages::InternalMessage::CollectorMessage(
            messages::CollectorMessage::GetURITemplate {
                uri: "https://www.google.com".to_string(),
                location: "tmp".to_string(),
            },
        );
        actor_ref.send(&message);
        assert!(true);
        let message = messages::InternalMessage::NoMessage;
        let _ = actor_ref.send(&message);
    }

    #[tokio::test]
    async fn test_blocking_actor_ref_error() {
        let (snd, _rec) = std::sync::mpsc::channel::<InternalMessage>();
        let falsesend1 = tokio::sync::mpsc::channel::<InternalMessage>(10).0;
        let falsesend2 = tokio::sync::mpsc::channel::<InternalMessage>(10).0;
        let (_guardsnd, _guardrec) = tokio::sync::mpsc::channel::<messages::GuardianMessage>(10);
        let actor = actor::Actor::NotAnActor;
        let guardian = actor::Actor::Guardian(Guardian::new(_guardrec));
        let collector = actor::Actor::Collector(actor::Collector::new(_rec));
        assert!(TokioActorRef::new(guardian, falsesend1).is_err());
        assert!(BlockingActorRef::new(actor, snd).is_err());
        assert!(TokioActorRef::new(collector, falsesend2).is_err());
    }
}

// grcov-excl-stop
