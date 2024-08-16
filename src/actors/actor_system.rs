
use std::io::{Error, ErrorKind};

use super::{actor::ActorTrait, actor_ref::GuardianActorRef, guardian::Guardian, messages::{ Message, ResponseMessage}};
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
    ///    actor_system.create_officer<T>(SelectActor::LogActor).await.unwrap();
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
        let msg = Message::Terminate;
         self.guardian_ref.send(msg).await;
    }
    pub async fn create_officer<T: ActorTrait>(&mut self, actor_type: T) -> Result<(), Error> {
        // Need to figure out how to get the actor to the officer. Should we create the actor ref here?)
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = Message::GetId; // TODO: change this to the correct message for creating an officer
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
        }
    }

    fn create_blocking_officer<T>(&mut self, actor_type: T) -> Result<(), Error> {
        // create actor reference here and send it to the guardian.

        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        /**
        
        let msg = GuardianMessage::CreateOfficer { officer_type: actor, responder: tx };
        self.guardian_ref.send(msg);
        match _rx {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
        }
        */

        Ok(())
    }

    pub async fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        // send message to guardian to remove officer
        /** 
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::RemoveOfficer { officer_id, responder: tx };
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
        }
        */
        Ok(())
    }

    pub async fn dispatch(&mut self, officer_id : u32, message: Message) {
        info!("Dispatching message to actor system");
        self.guardian_ref.send(Message::GetId).await; // convert to correct message type when available
    }

    pub async fn add_courrier<T: ActorTrait + 'static>(&mut self, officer_id: u32, courrier_type: T) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        // convert below into an actor reference when available then send to guardian / officer.
        //let actor = Actor::new(courrier_type, _rx);

        let msg = Message::GetId; // TODO: convert to correct message type when available
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
        }
    }

    pub async fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error> {
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = Message::GetId; // TODO: convert to correct message type when available
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
    
    }
}

// grcov-excl-stop