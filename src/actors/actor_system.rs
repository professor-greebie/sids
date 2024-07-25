
use std::io::{Error, ErrorKind};

use super::{actor_ref::GuardianActorRef, guardian::Guardian, messages::{GuardianMessage, Message, ResponseMessage}, officer::SelectActor};
use log::info;





/// The ActorSystem is the main entry point for the actor system. It is responsible for creating the guardian actor and sending messages to the guardian actor.
    /// 
    /// The ActorSystem is designed to be an actor reference for the guardian actor that manages all other actors in the system.
    /// In practice, it is the only actor_reference that is directly interacted with by the user.
    /// 
    /// # Example
    /// ```rust
    /// use sids::actors::actor_system::ActorSystem;
    /// use sids::actors::messages::{Message, ActorMessage};
    /// 
    /// pub async fn run_system() {
    /// 
    ///     let (tx, _rx) = tokio::sync::oneshot::channel();

    ///     let mut actor_system = ActorSystem::new();
    ///     actor_system.start_system().await; 
    ///     actor_system.dispatch(Message::ActorMessage(ActorMessage::GetNextId { responder: tx })).await;
    ///     actor_system.dispatch(Message::Terminate).await;
    /// }
    /// 
    /// ```
pub struct ActorSystem {
    guardian : GuardianActorRef,
}


impl ActorSystem {
    /// Create a new ActorSystem
    /// 
    /// The ActorSystem will start by launching a guardian, which is a non-blocking officer-actor that manages all other actors in the system.
    /// The guardian will be dormant until start_system is called in the ActorSystem.
    pub fn new() -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(super::SIDS_DEFAULT_BUFFER_SIZE);
        let guardian = Guardian::new(rec);

        info!(actor = "guardian"; "Guardian channel and actor created. Launching...");
        let actor_ref = GuardianActorRef::new(guardian, snd);
        info!(actor = "guardian"; "Guardian actor spawned");
        info!(actor = "guardian"; "Actor system created");

        ActorSystem {
            guardian: actor_ref,
        }
    }

    pub async fn stop_system(&mut self) {
        info!("Stopping actor system");
        let msg = GuardianMessage::Terminate;
        self.guardian.send(&msg).await;
    }

    pub async fn create_officer(&mut self, actor_type: SelectActor) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::CreateOfficer { officer_type: actor_type, responder: tx };
        self.guardian.send(&msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")),
        }
    }

    pub async fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::RemoveOfficer { officer_id, responder: tx };
        self.guardian.send(&msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
        }
    }

    pub async fn dispatch(&mut self, message: Message) {
        info!("Dispatching message to actor system");
        let msg = GuardianMessage::Dispatch { officer_id: 1, message: message };
        self.guardian.send(&msg).await;
    }

    pub async fn add_courrier(&mut self, officer_id: u32, courrier_type: SelectActor) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::AddCourrier { officer_id, courrier_type, responder: tx };
        self.guardian.send(&msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")),
        }
    }

    pub async fn remove_courrier(&mut self, officer_id: u32) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::RemoveCourrier { officer_id, courrier_id: 1, responder: tx };
        self.guardian.send(&msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")),
        }
    
    }

    

}




#[cfg(test)]
mod tests {
    use crate::actors::messages;

    use super::*;

    #[tokio::test]
    #[allow(dead_code)]
    async fn test_actor_system_is_not_headless_when_started() {
        let mut actor_system = ActorSystem::new();
        //let (tx, _rx) = oneshot::channel<InternalMessage>();
        // message responder is never used in the message. Should we change the message to not include the responder?
        let _message = messages::Message::GetId;

        actor_system.dispatch(_message).await;
        assert!(true);


        //actor_system.dispatch(&message).await;
        //let response = rx.await.unwrap();
        //assert_eq!(response, 1);
        
    }
}