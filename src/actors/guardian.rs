use log::info;
use std::io::Error;
use std::{io::ErrorKind, result::Result};
use tokio::sync::mpsc;

use super::actor::create_dummy_actor;
use super::actor_ref::{ActorRef, BlockingActorRef};
use super::messages::{GuardianMessage, Message, ResponseMessage};
use super::officer::BlockingOfficer;
use super::officer::Officer;

trait OfficerFactory {
    fn create_officer(&mut self, officer_type: ActorRef) -> Result<(), Error>;
    fn create_blocking_officer(&mut self, officer_type: BlockingActorRef) -> Result<(), Error>;
    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error>;
    fn add_courrier(&mut self, officer_id: u32, courrier_type: ActorRef) -> Result<(), Error>;
    fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error>;
}

pub(super) struct Guardian {
    pub(super) receiver: mpsc::Receiver<GuardianMessage>,
    pub(super) officers: Vec<Officer>,
    pub(super) blocking_officers: Vec<BlockingOfficer>,
}

impl Guardian {
    pub(super) fn new(receiver: tokio::sync::mpsc::Receiver<GuardianMessage>) -> Guardian {
        Guardian {
            receiver,
            officers: Vec::new(),
            blocking_officers: Vec::new(),
        }
    }

    pub(super) async fn receive(&mut self, message: GuardianMessage) {
        match message {
            GuardianMessage::CreateOfficer {
                officer_type,
                responder,
            } => {
                self.create_officer(officer_type)
                    .expect("Failed to create officer");
                responder
                    .send(ResponseMessage::Success)
                    .expect("Failed to send response");
            }
            GuardianMessage::CreateBlockingOfficer {
                officer_type,
                responder,
            } => {
                self.create_blocking_officer(officer_type)
                    .expect("Failed to create officer");
                responder
                    .send(ResponseMessage::Success)
                    .expect("Failed to send response");
            }
            GuardianMessage::RemoveOfficer {
                officer_id,
                responder,
            } => {
                self.remove_officer(officer_id)
                    .expect("Failed to remove officer");
                responder
                    .send(ResponseMessage::Success)
                    .expect("Failed to send response");
            }
            GuardianMessage::OfficerMessage {
                officer_id,
                message,
                blocking,
            } => {
                info!(
                    "Guardian received message to send message to {}",
                    officer_id
                );
                if blocking {
                    self.send_message_to_blocking_officer(officer_id, message)
                        .await;
                } else {
                    self.send_message_to_officer(officer_id, message).await;
                }
            }
            GuardianMessage::AddCourrier {
                officer_id,
                courrier_type,
                responder,
                blocking,
            } => {
                if blocking {
                    if let Some(blocking_officer) =
                        self.blocking_officers.get_mut(officer_id as usize)
                    {
                        blocking_officer.subscribe(courrier_type);
                    }
                } else {
                    self.add_courrier(officer_id, courrier_type)
                        .expect("Failed to add courrier");
                    responder
                        .send(ResponseMessage::Success)
                        .expect("Failed to send response");
                }
            }

            GuardianMessage::NotifyCourriers {
                officer_id,
                message,
                responder,
                blocking,
            } => {
                info!("Guardian received message: {:?}", message);

                // broadcast notifications require that actors can receive a cloneable message type or a borrowed message type
                // might not be necessary for the current implementation but can be build in if its desired.
                if blocking {
                    if let Some(blocking_officer) =
                        self.blocking_officers.get_mut(officer_id as usize)
                    {
                        blocking_officer.notify(&message).unwrap();
                    }
                } else if let Some(officer) = self.officers.get_mut(officer_id as usize) {
                    officer.notify(&message).unwrap();
                }
                responder
                    .send(ResponseMessage::Success)
                    .expect("Failed to send response");
            }
            GuardianMessage::RemoveCourrier {
                officer_id,
                courrier_id,
                responder,
                blocking,
            } => {
                if blocking {
                    if let Some(blocking_officer) =
                        self.blocking_officers.get_mut(officer_id as usize)
                    {
                        blocking_officer.unsubscribe(courrier_id);
                    }
                } else {
                    self.remove_courrier(officer_id, courrier_id)
                        .expect("Failed to remove courrier");
                    responder
                        .send(ResponseMessage::Success)
                        .expect("Failed to send response");
                }
            }
            GuardianMessage::Terminate => {
                info!("Guardian received terminate message");
                //self.stop_system().await;
            }
            GuardianMessage::CountOfficers { responder } => responder
                .send(ResponseMessage::ResponseCount(
                    "officers".to_string(),
                    self.officers.len() as u32,
                ))
                .expect("Failed to send response"),
            GuardianMessage::CountCourriers {
                officer_id,
                responder,
            } => responder
                .send(ResponseMessage::ResponseCount(
                    "courriers".to_string(),
                    self.count_courriers(officer_id),
                ))
                .expect("Failed to send response"),
            GuardianMessage::CountActors { responder } => responder
                .send(ResponseMessage::ResponseCount(
                    "actors".to_string(),
                    self.count_all_actors(),
                ))
                .expect("Failed to send response"),
        }
    }

    async fn send_message_to_officer(&mut self, officer_id: u32, message: Message) {
        info!("Sending message to officer {}", officer_id);
        if let Some(officer) = self.officers.get_mut(officer_id as usize) {
            officer.send(message).await;
        }
    }

    async fn send_message_to_blocking_officer(&mut self, officer_id: u32, message: Message) {
        info!("Sending message to blocking officer {}", officer_id);
        if let Some(blocking_officer) = self.blocking_officers.get_mut(officer_id as usize) {
            blocking_officer.send(message);
        }
    }

    fn count_all_actors(&self) -> u32 {
        let officers = (self.officers.len() + self.blocking_officers.len()) as u32;
        let courriers = self
            .officers
            .iter()
            .fold(0, |acc, officer| acc + officer.courriers.len() as u32);
        officers + courriers
    }

    fn count_courriers(&self, officer_id: u32) -> u32 {
        if let Some(officer) = self.officers.get(officer_id as usize) {
            return officer.courriers.len() as u32;
        }
        0
    }
}

impl OfficerFactory for Guardian {
    fn create_officer(&mut self, officer_type: ActorRef) -> Result<(), Error> {
        let officer_id = self.officers.len() as u32;
        let officer = Officer::new(officer_id, officer_type);
        self.officers.push(officer);
        Ok(())
    }

    fn create_blocking_officer(&mut self, officer_type: BlockingActorRef) -> Result<(), Error> {
        let officer_id = self.officers.len() as u32;
        let dummy_actor_ref = create_dummy_actor();
        let officer = Officer::new(officer_id, dummy_actor_ref);
        let blocking_officer = BlockingOfficer::new(officer, officer_type);
        self.blocking_officers.push(blocking_officer);
        Ok(())
    }

    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        let _ = match self.officers.get_mut(officer_id as usize) {
            Some(officer) => officer,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Officer found with id {}.", officer_id),
                ));
            }
        };
        // do we terminate the officer here?
        //officer_to_remove.send( GuardianMessage::Terminate).await;
        self.officers.remove(officer_id as usize);
        Ok(())
    }

    fn add_courrier(&mut self, officer_id: u32, courrier: ActorRef) -> Result<(), Error> {
        let officer = match self.officers.get_mut(officer_id as usize) {
            Some(officer) => officer,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Officer found with id {}.", officer_id),
                ));
            }
        };
        officer.subscribe(courrier);
        Ok(())
    }

    fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error> {
        let officer = match self.officers.get_mut(officer_id as usize) {
            Some(officer) => officer,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Officer found with id {}.", officer_id),
                ));
            }
        };
        match officer.courriers.get(courrier_id as usize) {
            Some(_) => (),
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Courrier found with id {}.", courrier_id),
                ));
            }
        };

        officer.unsubscribe(courrier_id);

        Ok(())
    }
}

// grcov-excl-start
#[cfg(test)]
mod tests {
    use crate::actors::actor::Actor;
    use crate::actors::actor::ActorImpl;
    use crate::actors::messages;

    use super::*;
    use messages::ResponseMessage;
    use tokio::sync::oneshot;

    use tokio::sync::mpsc;

    struct SampleActor;
    impl Actor for SampleActor {
        fn receive(&mut self, _message: Message) {
            // do nothing
            info!("Received message");
            if let Message::StringMessage { message } = _message {
                info!("Received message: {}", message);
            }
        }
    }

    #[tokio::test]
    async fn test_guardian_actor() {
        let (_tx, rx) = mpsc::channel(1);
        let (actor_tx, actor_rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let actor = ActorImpl::new(None, SampleActor, actor_rx);
        let officer = ActorRef::new(actor, actor_tx);
        let (rec_tx, rec_rx) = oneshot::channel::<ResponseMessage>();
        guardian
            .receive(GuardianMessage::CreateOfficer {
                officer_type: officer,
                responder: rec_tx,
            })
            .await;
        let response = rec_rx.await.unwrap();
        assert_eq!(response, ResponseMessage::Success);
    }

    #[tokio::test]
    async fn test_guardian_actor_count_actors() {
        let (_tx, rx) = mpsc::channel(1);
        let (off_tx, off_rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let (rec_tx, rec_rx) = oneshot::channel::<ResponseMessage>();
        let (rec2_tx, rec2_rx) = oneshot::channel::<ResponseMessage>();
        let actor = ActorImpl::new(None, SampleActor, off_rx);
        let officer = ActorRef::new(actor, off_tx);
        guardian
            .receive(GuardianMessage::CreateOfficer {
                officer_type: officer,
                responder: rec_tx,
            })
            .await;
        guardian
            .receive(GuardianMessage::CountActors { responder: rec2_tx })
            .await;
        let response = rec_rx.await.unwrap();
        assert_eq!(response, ResponseMessage::Success);
        let response2 = rec2_rx.await.unwrap();
        assert_eq!(
            response2,
            ResponseMessage::ResponseCount("actors".to_string(), 1)
        );
    }

    #[tokio::test]
    async fn test_guardian_send_features() {
        /* let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let (tx, rx) = oneshot::channel::<ResponseMessage>(); */
    }

    #[tokio::test]
    async fn test_guardian_actor_creates_courriers() {
        /* let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx); */
    }
}

// grcov-excl-stop
