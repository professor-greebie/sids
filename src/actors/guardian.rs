use log::info;
use std::io::Error;
use std::{io::ErrorKind, result::Result};
use tokio::sync::mpsc;

use super::{
    actor,
    actor_ref::{self, BlockingActorRef, TokioActorRef},
    messages,
    officer::{Officer, SelectActor},
};

pub trait OfficerFactory {
    fn create_officer(&mut self, officer_type: SelectActor) -> Result<(), Error>;
    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error>;
    fn add_courrier(&mut self, officer_id: u32, courrier_type: SelectActor) -> Result<(), Error>;
    fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error>;
}

pub(super) struct Guardian {
    receiver: mpsc::Receiver<messages::GuardianMessage>,
    officers: Vec<Officer>,
}

impl Guardian {
    async fn receive(&mut self, message: messages::GuardianMessage) -> Result<(), Error> {
        // Will officers always be non-blocking?
        match message {
            messages::GuardianMessage::Terminate => {
                for officer in self.officers.iter_mut() {
                    officer.send(messages::Message::Terminate).await;
                }
                // TODO: Terminate all officers and their courriers.
                // Do we send a response here?
                info!(actor="guardian";"Guardian Actor terminated");
            }
            messages::GuardianMessage::NoMessage => {
                info!(actor="guardian"; "No message to send");
            }
            messages::GuardianMessage::Dispatch {
                officer_id,
                message,
            } => {
                for officer in self.officers.iter_mut() {
                    if officer._id == officer_id {
                        officer.send(message.clone()).await;
                    }
                }
            }
            messages::GuardianMessage::CreateOfficer {
                officer_type,
                responder,
            } => {
                // this should use the create_officer function where possible.
                let officer_id = self.officers.len() as u32;
                match officer_type {
                    SelectActor::Collector => {
                        let (blocking_snd, blocking_rec) =
                            std::sync::mpsc::channel::<messages::InternalMessage>();
                        let collector_actor =
                            actor::Actor::Collector(actor::Collector::new(blocking_rec));
                        let _ = match BlockingActorRef::new(collector_actor, blocking_snd) {
                            Ok(actor_ref) => {
                                let officer = Officer {
                                    _id: officer_id,
                                    _type: SelectActor::Collector,
                                    actor: actor_ref::ActorRef::BlockingActorRef(actor_ref),
                                    courriers: Vec::new(),
                                };
                                self.officers.push(officer);
                                responder.send(messages::ResponseMessage::Success).unwrap();
                            }
                            Err(_) => {
                                responder.send(messages::ResponseMessage::Failure).unwrap();
                            }
                        };
                    }
                    SelectActor::CleaningActor => {
                        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
                            super::SIDS_DEFAULT_BUFFER_SIZE,
                        );
                        let actor = actor::Actor::CleaningActor(actor::CleaningActor::new(rec));
                        let _ = match TokioActorRef::new(actor, snd) {
                            Ok(actor_ref) => {
                                let officer = Officer {
                                    _id: officer_id,
                                    _type: SelectActor::CleaningActor,
                                    actor: actor_ref::ActorRef::TokioActorRef(actor_ref),
                                    courriers: Vec::new(),
                                };
                                self.officers.push(officer);
                                responder.send(messages::ResponseMessage::Success).unwrap();
                            }
                            Err(_) => {
                                responder.send(messages::ResponseMessage::Failure).unwrap();
                            }
                        };
                    }
                    SelectActor::KafkaProducerActor => {
                        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
                            super::SIDS_DEFAULT_BUFFER_SIZE,
                        );
                        // what to do if port and/or host are specified?
                        let actor = actor::Actor::KafkaProducerActor(
                            actor::KafkaProducerActor::new(rec, None, None),
                        );
                        let _ = match TokioActorRef::new(actor, snd) {
                            Ok(actor_ref) => {
                                let officer = Officer {
                                    _id: officer_id,
                                    _type: SelectActor::KafkaProducerActor,
                                    actor: actor_ref::ActorRef::TokioActorRef(actor_ref),
                                    courriers: Vec::new(),
                                };
                                self.officers.push(officer);
                                responder.send(messages::ResponseMessage::Success).unwrap();
                            }
                            Err(_) => {
                                responder.send(messages::ResponseMessage::Failure).unwrap();
                            }
                        };
                    }
                    SelectActor::LogActor => {
                        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
                            super::SIDS_DEFAULT_BUFFER_SIZE,
                        );
                        let actor = actor::Actor::LogActor(actor::LogActor::new(rec));
                        let _ = match TokioActorRef::new(actor, snd) {
                            Ok(actor_ref) => {
                                let officer = Officer {
                                    _id: officer_id,
                                    _type: SelectActor::LogActor,
                                    actor: actor_ref::ActorRef::TokioActorRef(actor_ref),
                                    courriers: Vec::new(),
                                };
                                self.officers.push(officer);
                                responder.send(messages::ResponseMessage::Success).unwrap();
                            }
                            Err(_) => {
                                responder.send(messages::ResponseMessage::Failure).unwrap();
                            }
                        };
                    }
                    _ => {
                        responder.send(messages::ResponseMessage::Failure).unwrap();
                    }
                }
            }
            messages::GuardianMessage::RemoveOfficer {
                officer_id,
                responder,
            } => match self.remove_officer(officer_id) {
                Ok(_) => {
                    responder.send(messages::ResponseMessage::Success).unwrap();
                }
                Err(_) => {
                    responder.send(messages::ResponseMessage::Failure).unwrap();
                }
            },
            messages::GuardianMessage::AddCourrier {
                officer_id,
                courrier_type,
                responder,
            } => match self.add_courrier(officer_id, courrier_type) {
                Ok(_) => {
                    responder.send(messages::ResponseMessage::Success).unwrap();
                }
                Err(_) => {
                    responder.send(messages::ResponseMessage::Failure).unwrap();
                }
            },
            messages::GuardianMessage::RemoveCourrier {
                officer_id,
                courrier_id,
                responder,
            } => match self.remove_courrier(officer_id, courrier_id) {
                Ok(_) => {
                    responder.send(messages::ResponseMessage::Success).unwrap();
                }
                Err(_) => {
                    responder.send(messages::ResponseMessage::Failure).unwrap();
                }
            },

            #[allow(unreachable_patterns)]
            _ => {
                info!(actor="guardian"; "No message to send");
            }
        }

        Ok(())
    }

    pub(super) fn new(receiver: mpsc::Receiver<messages::GuardianMessage>) -> Guardian {
        Guardian {
            receiver: receiver,
            officers: Vec::new(),
        }
    }

    pub(super) async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

impl OfficerFactory for Guardian {
    fn create_officer(&mut self, officer_type: SelectActor) -> Result<(), Error> {
        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
            super::SIDS_DEFAULT_BUFFER_SIZE,
        );
        let (blocking_snd, blocking_rec) = std::sync::mpsc::channel::<messages::InternalMessage>();
        match officer_type {
            SelectActor::Guardian => Err(Error::new(
                ErrorKind::InvalidInput,
                "Cannot create guardian officer outside of ActorSystem.",
            )),
            SelectActor::Collector => {
                let collector_officer = Officer {
                    _id: 1,
                    _type: SelectActor::Collector,
                    actor: actor_ref::ActorRef::BlockingActorRef(BlockingActorRef::new(
                        actor::Actor::Collector(actor::Collector::new(blocking_rec)),
                        blocking_snd,
                    )?),
                    courriers: Vec::new(),
                };
                self.officers.push(collector_officer);
                return Ok(());
            }
            SelectActor::CleaningActor => {
                let cleaning_officer = Officer {
                    _id: 2,
                    _type: SelectActor::CleaningActor,
                    actor: actor_ref::ActorRef::TokioActorRef(
                        TokioActorRef::new(
                            actor::Actor::CleaningActor(actor::CleaningActor::new(rec)),
                            snd,
                        )
                        .expect("Failed to create tokio actor ref"),
                    ),
                    courriers: Vec::new(),
                };
                self.officers.push(cleaning_officer);
                return Ok(());
            }
            SelectActor::KafkaProducerActor => {
                // need perhaps to include the host and the port for kafka
                let kafka_actor = Officer {
                    _id: 4,
                    _type: SelectActor::KafkaProducerActor,
                    actor: actor_ref::ActorRef::TokioActorRef(
                        TokioActorRef::new(
                            actor::Actor::KafkaProducerActor(actor::KafkaProducerActor::new(
                                rec, None, None,
                            )),
                            snd,
                        )
                        .expect("Failed to create tokio actor ref"),
                    ),
                    courriers: Vec::new(),
                };
                self.officers.push(kafka_actor);
                return Ok(());
            }
            SelectActor::LogActor => {
                let log_actor = Officer {
                    _id: 3,
                    _type: SelectActor::LogActor,
                    actor: actor_ref::ActorRef::TokioActorRef(
                        TokioActorRef::new(actor::Actor::LogActor(actor::LogActor::new(rec)), snd)
                            .expect("Failed to create tokio actor ref"),
                    ),
                    courriers: Vec::new(),
                };
                self.officers.push(log_actor);
                return Ok(());
            }
        }
    }

    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        for (index, officer) in self.officers.iter().enumerate() {
            if officer._id == officer_id {
                self.officers.remove(index);
                return Ok(());
            }
        }
        Err(Error::new(ErrorKind::NotFound, "Officer not found."))
    }

    fn add_courrier(&mut self, officer_id: u32, courrier_type: SelectActor) -> Result<(), Error> {
        let officer = self.officers.get_mut(officer_id as usize).unwrap();
        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
            super::SIDS_DEFAULT_BUFFER_SIZE,
        );
        match courrier_type {
            SelectActor::Guardian => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Cannot create guardian officer outside of ActorSystem.",
                ));
            },
            SelectActor::CleaningActor => {
                let actor = actor::Actor::CleaningActor(actor::CleaningActor::new(rec));
                match actor_ref::TokioActorRef::new(actor, snd) {
                    Ok(actor_ref) => {
                        officer.subscribe(actor_ref::ActorRef::TokioActorRef(actor_ref));
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Failed to create actor ref.",
                        ));
                    }
                }
            },
            SelectActor::Collector => {
                let (blocking_snd, blocking_rec) =
                    std::sync::mpsc::channel::<messages::InternalMessage>();
                let actor = actor::Actor::Collector(actor::Collector::new(blocking_rec));
                match actor_ref::BlockingActorRef::new(actor, blocking_snd) {
                    Ok(actor_ref) => {
                        officer.subscribe(actor_ref::ActorRef::BlockingActorRef(actor_ref));
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Failed to create actor ref.",
                        ));
                    }
                }
            },
            SelectActor::KafkaProducerActor => {
                let actor = actor::Actor::KafkaProducerActor(actor::KafkaProducerActor::new(
                    rec, None, None,
                ));
                match actor_ref::TokioActorRef::new(actor, snd) {
                    Ok(actor_ref) => {
                        officer.subscribe(actor_ref::ActorRef::TokioActorRef(actor_ref));
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Failed to create actor ref.",
                        ));
                    }
                }
            },
            SelectActor::LogActor => {
                let actor = actor::Actor::LogActor(actor::LogActor::new(rec));
                let _ = match actor_ref::TokioActorRef::new(actor, snd) {
                    Ok(actor_ref) => {
                        officer.subscribe(actor_ref::ActorRef::TokioActorRef(actor_ref));
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Failed to create actor ref.",
                        ));
                    }
                };
            },
        }
    }

    fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error> {
        for officer in self.officers.iter_mut() {
            if officer._id == officer_id {
                officer.unsubscribe(courrier_id);
            }
        }
        Err(Error::new(ErrorKind::NotFound, "Officer not found."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messages::ResponseMessage;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_guardian_actor() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        //guardian.run().await;
        guardian
            .create_officer(SelectActor::Collector)
            .expect("Failed to create officer.");
        guardian
            .create_officer(SelectActor::CleaningActor)
            .expect("Failed to create officer.");
        guardian
            .create_officer(SelectActor::KafkaProducerActor)
            .expect("Failed to create officer.");
        // Let's test this properly.
        guardian
            .create_officer(SelectActor::LogActor)
            .expect("Failed to create officer.");
        assert!(guardian.officers.len() == 4);
        guardian
            .remove_officer(1)
            .expect("Failed to remove officer.");

        guardian
            .receive(messages::GuardianMessage::Terminate)
            .await
            .expect("Failed to terminate guardian.");
        guardian
            .create_officer(SelectActor::Guardian)
            .expect_err("Cannot create guardian officer outside of ActorSystem.");
    }

    #[tokio::test]
    async fn test_guardian_actor_get_next_id() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let (tx, rx) = oneshot::channel();
        guardian
            .receive(messages::GuardianMessage::CreateOfficer {
                officer_type: SelectActor::LogActor,
                responder: tx,
            })
            .await
            .expect("Failed to get next id.");
        assert_eq!(rx.await.unwrap(), ResponseMessage::Success);
    }


}
