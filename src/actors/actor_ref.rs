use std::io::Error;

use log::{error, info };

use crate::actors::{actor, messages::InternalMessage, guardian::Guardian};

use super::messages::{self, ResponseMessage};

type TokioSender = tokio::sync::mpsc::Sender<messages::InternalMessage>;
type StdSender = std::sync::mpsc::Sender<messages::InternalMessage>;
type GuardianSender = tokio::sync::mpsc::Sender<messages::GuardianMessage>;

#[allow(dead_code)]
pub (super) enum ActorRef {
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

    pub(super) async fn send(&mut self, message: &messages::GuardianMessage) {
        match message {
            
            messages::GuardianMessage::Terminate => {
                info!(actor = "Guardian"; "Terminating guardian actor");
                let _ = self.sender.send(messages::GuardianMessage::Terminate).await;
            },
            messages::GuardianMessage::Dispatch { officer_id, message }  => {
                info!(actor = "Guardian"; "Dispatching message to officer");
                let _ = self.sender.send(messages::GuardianMessage::Dispatch { officer_id: *officer_id, message: message.clone() }).await;
            },
            _ => {
                error!("No message to send");
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
                return Err(Error::new(std::io::ErrorKind::InvalidInput, "Actor not found."));
            }
        }
        
        Ok(Self { sender })
    }

    pub(super) fn send(&mut self, message: &messages::InternalMessage) {
        let (snd, rec) = std::sync::mpsc::channel();
        match message {
            messages::InternalMessage::CollectorMessage(messages::CollectorMessage::GetURITemplate {
                uri,
                location,
            }) => {
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
                    messages::ResponseMessage::Response(messages::Response::Success) => {
                        info!("Success");
                    }
                    messages::ResponseMessage::Response(messages::Response::Failure) => {
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
    pub(super) fn new(actor: actor::Actor, snd: TokioSender) -> Self {
        match actor {
            actor::Actor::Guardian(guardian) => {
                info!(actor = "Guardian"; "Spawning guardian actor");
                tokio::spawn(async move {
                    let mut guardian = guardian;
                    guardian.run().await;
                });
            },
            actor::Actor::KafkaProducerActor(kafka_actor) => {
                info!(actor = "Kafka Producer"; "Spawning a Kafka Producing actor");
                tokio::spawn(async move {
                    let mut kafka_actor = kafka_actor;
                    kafka_actor.run().await;
                });
            },
            _ => {
                error!("Actor not found");
            }
        }
        Self { sender: snd }
    }

    pub(super) async fn send(&mut self, message: &messages::InternalMessage) {
        match message {
            messages::InternalMessage::KafkaProducerMessage(messages::KafkaProducerMessage::Produce {
                topic,
                key,
                message,
                responder: _,
            }) => {
                info!(actor = "Kafka Producer"; "Producing Kafka message");
                let (snd, _rec) = tokio::sync::oneshot::channel::<ResponseMessage>();
                let _ = self
                    .sender
                    .send(messages::InternalMessage::KafkaProducerMessage(
                        messages::KafkaProducerMessage::Produce {
                            topic: topic.to_string(),
                            key: key.to_string(),
                            message: message.to_string(),
                            responder: snd,
                        },
                    ))
                    .await;
                _rec.await.expect("Actor was killed before send.");
            },
            messages::InternalMessage::ActorMessage(messages::ActorMessage::GetNextId) => {

                // send the responder or force the actor reference to send the response?
                let (_snd, _rec) = tokio::sync::oneshot::channel::<ResponseMessage>();
                let _ = self.sender.send(messages::InternalMessage::ActorMessage(messages::ActorMessage::GetNextId)).await;
                // do something with the id;
                _rec.await.expect("Actor was killed before send.");
            },
            messages::InternalMessage::Terminate => {
                let _ = self.sender.send(messages::InternalMessage::Terminate).await;
            },
            _ => {
                let _ = self.sender.send(messages::InternalMessage::NoMessage).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use actor::LogActor;


    


    use super::*;



    #[tokio::test]
    async fn test_tokio_actor_ref() {
        let (snd, rec) = tokio::sync::mpsc::channel::<InternalMessage>(10);
        let (_snd2, _rec2) = tokio::sync::oneshot::channel::<u64>();
        let actor = actor::Actor::LogActor(LogActor::new(rec));
        let mut actor_ref = TokioActorRef::new(actor, snd);
        // the message never uses the responder here. Should we change the message to not include the responder?
        let message = messages::InternalMessage::ActorMessage(messages::ActorMessage::GetNextId);
        let _ = actor_ref.send(&message).await;
        let no_message = messages::InternalMessage::NoMessage;
        let _ = actor_ref.send(&no_message).await;
        assert!(true);
    }

    #[test]
    fn test_blocking_actor_ref() {
        let (snd, rec) = std::sync::mpsc::channel::<InternalMessage>();
        let actor = actor::Actor::Collector(actor::Collector::new(rec));
        let mut actor_ref = BlockingActorRef::new(actor, snd).unwrap();
        let message = messages::InternalMessage::CollectorMessage(messages::CollectorMessage::GetURITemplate {
            uri: "https://www.google.com".to_string(),
            location: "tmp".to_string(),
        });
        actor_ref.send(&message);
        assert!(true);
        let message = messages::InternalMessage::NoMessage;
        let _ = actor_ref.send(&message);

    }

    #[test]
    fn test_blocking_actor_ref_error() {
        let (snd, _rec) = std::sync::mpsc::channel();
        let actor = actor::Actor::NotAnActor;
        assert!(BlockingActorRef::new(actor, snd).is_err());
    }
}
