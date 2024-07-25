
use std::{io::ErrorKind, result::Result};
use std::io::Error;
use log::info;
use tokio::sync::mpsc;

use super::{actor, actor_ref::{self, BlockingActorRef, TokioActorRef}, messages, officer::{Officer, SelectActor}};

pub trait OfficerFactory {
    fn create_officer(&mut self, officer_type: SelectActor) -> Result<(), Error>;
    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error>;
    fn add_courrier(&mut self, officer_id: u32, courrier_type: SelectActor) -> Result<(), Error>;
}


pub(super) struct Guardian {
    receiver: mpsc::Receiver<messages::GuardianMessage>,
    officers: Vec<Officer>,
}

impl  Guardian {
    async fn receive(&mut self, message: messages::GuardianMessage) -> Result<(), Error> {
        // Will officers always be non-blocking?
        match message {
            messages::GuardianMessage::Terminate => {
                for officer in self.officers.iter_mut() {
                    officer.send(messages::Message::Terminate).await;
                }
                // TODO: Terminate all officers and their courriers.
                info!("Guardian Actor terminated");
            },
            messages::GuardianMessage::Dispatch { officer_id, message } => {
                for officer in self.officers.iter_mut() {
                    if officer._id == officer_id {
                        officer.send(message.clone()).await;
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    pub(super) fn new(receiver: mpsc::Receiver<messages::GuardianMessage>) -> Guardian {
        Guardian { receiver: receiver, officers: Vec::new() }
    }

    pub(super) async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.receive(message).await.unwrap();
        }
    }
}

impl OfficerFactory for Guardian {
     fn create_officer(&mut self, officer_type: SelectActor) -> Result<(), Error> {
        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(super::SIDS_DEFAULT_BUFFER_SIZE);
        let (blocking_snd, blocking_rec) = std::sync::mpsc::channel::<messages::InternalMessage>();
        match officer_type {
            SelectActor::Guardian => Err(Error::new(ErrorKind::InvalidInput, "Cannot create guardian officer outside of ActorSystem.")),
            SelectActor::Collector => {
                let collector_officer = Officer {
                    _id: 1,
                    _type: SelectActor::Collector,
                    actor: actor_ref::ActorRef::BlockingActorRef(BlockingActorRef::new(actor::Actor::Collector(actor::Collector::new(blocking_rec)), blocking_snd)?),
                    courriers: Vec::new(),
                };
                self.officers.push(collector_officer);
                return Ok(());
            },
            SelectActor::CleaningActor => {
                let cleaning_officer = Officer {
                    _id: 2,
                    _type: SelectActor::CleaningActor,
                    actor: actor_ref::ActorRef::TokioActorRef(TokioActorRef::new(actor::Actor::CleaningActor(actor::CleaningActor::new(rec)), snd)),
                    courriers: Vec::new(),
                };
                self.officers.push(cleaning_officer);
                return Ok(())

            },
            SelectActor::KafkaProducerActor => Ok(()),
            SelectActor::LogActor => Ok(()),
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

    fn add_courrier(&mut self, _officer_id: u32, _courrier_type: SelectActor) -> Result<(), Error> {
        Err(Error::new(ErrorKind::InvalidInput, "Not implemented."))
        
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
        guardian.create_officer(SelectActor::Collector).expect("Failed to create officer.");
        guardian.create_officer(SelectActor::CleaningActor).expect("Failed to create officer.");
        guardian.create_officer(SelectActor::KafkaProducerActor).expect("Failed to create officer.");
        guardian.create_officer(SelectActor::LogActor).expect("Failed to create officer.");
        assert!(guardian.officers.len() == 2);
        guardian.remove_officer(1).expect("Failed to remove officer.");

        guardian.receive(messages::GuardianMessage::Terminate).await.expect("Failed to terminate guardian.");
        guardian.create_officer(SelectActor::Guardian).expect_err("Cannot create guardian officer outside of ActorSystem.");

    }

    #[tokio::test]
    async fn test_guardian_actor_get_next_id() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let (tx, rx) = oneshot::channel();
        guardian.receive(messages::GuardianMessage::CreateOfficer { officer_type: SelectActor::LogActor, responder: tx}).await.expect("Failed to get next id.");
        assert_eq!(rx.await.unwrap(), ResponseMessage::Response(messages::Response::Success));
    }

}