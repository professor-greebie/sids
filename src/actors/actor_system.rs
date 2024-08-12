
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
    /// use sids::actors::messages::{Message, ResponseMessage};
    /// use sids::actors::officer::SelectActor;
    /// 
    /// pub async fn run_system() {
    /// 
    ///     let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();

    ///     let mut actor_system = ActorSystem::new();
    ///     
    ///    actor_system.create_officer(SelectActor::LogActor).await.unwrap();
    ///    actor_system.dispatch(0, Message::GetId).await;
    ///     actor_system.dispatch(0, Message::Terminate).await;
    /// }
    /// 
    /// ```
pub struct ActorSystem {
    guardian_ref : GuardianActorRef,
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
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
            guardian_ref: actor_ref,
        }
    }
    pub async fn stop_system(&mut self) {
        info!("Stopping actor system");
        let msg = GuardianMessage::Terminate;
         self.guardian_ref.send(msg).await;
    }
    pub async fn create_officer(&mut self, actor_type: SelectActor) -> Result<(), Error> {

        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::CreateOfficer { officer_type: actor_type, responder: tx };
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
        }
    }

    pub async fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::RemoveOfficer { officer_id, responder: tx };
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
        }
    }

    pub async fn dispatch(&mut self, officer_id : u32, message: Message) {
        info!("Dispatching message to actor system");
        self.guardian_ref.send(GuardianMessage::Dispatch { officer_id, message }).await;
    }

    pub async fn add_courrier(&mut self, officer_id: u32, courrier_type: SelectActor) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::AddCourrier { officer_id, courrier_type, responder: tx };
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
        }
    }

    pub async fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::RemoveCourrier { officer_id, courrier_id, responder: tx };
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")), // grcov-excl-line
        }
    
    }

    

}


// grcov-excl-start

#[cfg(test)]
mod tests {

    use crate::actors::messages;

    use super::*;

    #[tokio::test] 
    async fn test_actor_system_started() {
        let mut actor_system = ActorSystem::new();
        //let (tx, _rx) = oneshot::channel<InternalMessage>();
        // message responder is never used in the message. Should we change the message to not include the responder?
        let _message = messages::Message::GetId;

        
        actor_system.create_officer(SelectActor::Logging).await.unwrap();
        actor_system.dispatch(0, _message).await;
        assert!(actor_system.remove_officer(100).await.is_err());
        assert!(actor_system.create_officer(SelectActor::Logging).await.is_ok());
        assert!(actor_system.create_officer(SelectActor::Collector).await.is_ok());
        assert!(actor_system.create_officer(SelectActor::Cleaner).await.is_ok());
        assert!(actor_system.create_officer(SelectActor::Guardian).await.is_err());
        assert!(actor_system.remove_courrier(0, 0).await.is_err());
        assert!(actor_system.add_courrier(0, SelectActor::Collector).await.is_ok());
        assert!(actor_system.remove_courrier(0, 0).await.is_ok());
        actor_system.remove_officer(0).await.unwrap();
        //assert!(test);


        
        //assert!(true);
        actor_system.stop_system().await;
        assert!(true);



        //actor_system.dispatch(&message).await;
        //let response = rx.await.unwrap();
        //assert_eq!(response, 1);
        
    }
}

// grcov-excl-stop