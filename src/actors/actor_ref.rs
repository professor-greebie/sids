use log::{error, info };

use crate::actors::{actor, messages::Message};

use super::messages;

type TokioSender = tokio::sync::mpsc::Sender<messages::Message>;
type StdSender = std::sync::mpsc::Sender<messages::Message>;

#[allow(dead_code)]
pub (super) enum ActorRef {
    TokioActorRef(TokioActorRef),
    BlockingActorRef(BlockingActorRef),
}


#[allow(dead_code)]
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
        let (snd, rec) = std::sync::mpsc::channel();
        match message {
            messages::Message::CollectorMessage(messages::CollectorMessage::GetURITemplate {
                uri,
                location,
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
            },
            messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder: _resp }) => {

                // send the responder or force the actor reference to send the response?
                let (snd, _rec) = tokio::sync::oneshot::channel();
                let _ = self.sender.send(messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder: snd })).await;
                // do something with the id;
                _rec.await.expect("Actor was killed before send.");
            },
            messages::Message::Terminate => {
                let _ = self.sender.send(messages::Message::Terminate).await;
            },
            _ => {
                let _ = self.sender.send(messages::Message::NoMessage).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actors::guardian::Guardian;

    use super::*;

    #[tokio::test]
    async fn test_tokio_actor_ref() {
        let (snd, rec) = tokio::sync::mpsc::channel::<Message>(10);
        let (snd2, _rec2) = tokio::sync::oneshot::channel::<u64>();
        let actor = actor::Actor::Guardian(Guardian::new(rec));
        let mut actor_ref = TokioActorRef::new(actor, snd);
        // the message never uses the responder here. Should we change the message to not include the responder?
        let message = messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder: snd2 });
        let _ = actor_ref.send(&message).await;
    }

    #[test]
    fn test_blocking_actor_ref() {
        let (snd, rec) = std::sync::mpsc::channel();
        let actor = actor::Actor::Collector(actor::Collector::new(rec));
        let mut actor_ref = BlockingActorRef::new(actor, snd);
        let message = messages::Message::CollectorMessage(messages::CollectorMessage::GetURITemplate {
            uri: "https://www.google.com".to_string(),
            location: "tmp".to_string(),

        });
        actor_ref.send(&message);
        assert!(true);
    }
}
