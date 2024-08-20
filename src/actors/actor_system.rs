
use std::io::{Error, ErrorKind};

use super::{actor::{Actor, ActorTrait, BlockingActor, BlockingActorTrait}, actor_ref::{ActorRef, BlockingActorRef, GuardianActorRef}, guardian::Guardian, messages::{ InternalMessage, Message, ResponseMessage}};
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
    pub (super) async fn stop_system(&mut self) {
        info!("Stopping actor system");
        let msg = Message::Terminate;
         self.guardian_ref.send(msg).await;
    }
    pub (super) async fn create_officer<T: ActorTrait + 'static>(&mut self, name: Option<String>, actor_type: T) -> Result<(), Error> {
        info!("Creating officer called {}", name.clone().unwrap_or("Unnamed".to_string()));
        // Need to figure out how to get the actor to the officer. Should we create the actor ref here?)
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let (tx2, rx2) = tokio::sync::mpsc::channel::<InternalMessage>(super::SIDS_DEFAULT_BUFFER_SIZE);
        let actor = Actor::new(name, actor_type, rx2);
        let actor_ref = ActorRef::new(actor, tx2);
        let msg = Message::CreateOfficer { officer_type: actor_ref, responder: tx  }; // TODO: change this to the correct message for creating an officer
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
        }
    }

    pub (super) async fn create_blocking_officer<T: BlockingActorTrait + 'static>(&mut self, actor_type: T, name: Option<String>) -> Result<(), Error> {
        info!("Creating blocking officer called {}", name.clone().unwrap_or("Unnamed".to_string()));
        let (tx2, rx2) = std::sync::mpsc::channel::<InternalMessage>();
        // create actor reference here and send it to the guardian.
        let (tx, rx) = std::sync::mpsc::channel::<ResponseMessage>();
        let actor = BlockingActor::new(actor_type, rx2);
        let actor_ref = BlockingActorRef::new(actor, tx2);
        // send message to guardian to create officer
        let msg = Message::CreateBlockingOfficer { officer_type: actor_ref, responder: tx };
        self.guardian_ref.send(msg).await;
        let _ = match rx.recv() {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")),
        };
        Ok(())
    }

    pub async fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        // send message to guardian to remove officer
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = Message::RemoveOfficer { officer_id, responder: tx };
        self.guardian_ref.send(msg).await;
       let _ = match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
        };
        Ok(())
    }

    pub async fn dispatch(&mut self, officer_id : u32, message: InternalMessage, blocking: bool) {
        info!("Dispatching message to actor system");
        if let InternalMessage::StringMessage { message } = message {
            info!("Dispatching message: {}", message);
            self.guardian_ref.send(Message::OfficerMessage { officer_id, message: message, blocking }).await;
        }
         // convert to correct message type when available
    }

    pub async fn add_courrier<T: ActorTrait + 'static>(&mut self, officer_id: u32, courrier_type: T, name: Option<String>, blocking: bool) -> Result<(), Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let (tx2, rx2) = tokio::sync::mpsc::channel::<InternalMessage>(super::SIDS_DEFAULT_BUFFER_SIZE);
        let actor = Actor::new(name, courrier_type, rx2);
        let actor_ref = ActorRef::new(actor, tx2);
        let msg = Message::AddCourrier { officer_id: officer_id , courrier_type: actor_ref, responder: tx, blocking }; // TODO: convert to correct message type when available
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
        }
    }

    pub async fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32, blocking: bool) -> Result<(), Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = Message::RemoveCourrier { officer_id: officer_id, courrier_id: courrier_id, responder: tx, blocking }; // TODO: convert to correct message type when available
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")), // grcov-excl-line
        }
    }

    pub async fn notify_courriers(&mut self, officer_id: u32, message: InternalMessage, blocking: bool) -> Result<(), Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = Message::NotifyCourriers { officer_id, message, responder: tx, blocking };
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to notify courriers")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to notify courriers")), // grcov-excl-line
        }
    }
}


// grcov-excl-start

#[cfg(test)]
mod tests {


    use super::*;

    #[tokio::test] 
    async fn test_actor_system_started() {
        let _actor_system = ActorSystem::new();
    
    }
}

// grcov-excl-stop