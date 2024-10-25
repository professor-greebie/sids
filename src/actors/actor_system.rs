
use std::io::{Error, ErrorKind};

use crate::actors::actor::{ActorImpl, BlockingActorImpl};

use super::{actor::Actor, actor_ref::{ActorRef, BlockingActorRef}, channel_factory::ChannelFactory, guardian::Guardian, messages::{  ActorType, GuardianMessage, Mediator, Message, ResponseMessage}, SIDS_DEFAULT_BUFFER_SIZE};
use log::info;
use tokio::sync::{mpsc, oneshot};




/// The ActorSystem is the main entry point for the actor system. It is responsible for creating the guardian actor and sending messages to the guardian actor.
    /// 
    /// The ActorSystem is designed to be an actor reference for the guardian actor that manages all other actors in the system.
    /// In practice, it is the only actor_reference that is directly interacted with by the user.
    /// 
    /// # Example
    /// ```rust
    /// use sids::actors::actor_system::ActorSystem;
    /// use sids::actors::messages::{Message, ResponseMessage};
    /// 
    /// pub async fn run_system() {
    /// 
    ///     let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    /// }
    /// 
    /// ```
pub struct ActorSystem {
    guardian_ref : ActorRef<Message<ActorType, GuardianMessage>>,
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelFactory for ActorSystem {

    fn create_actor_channel(&self) -> (tokio::sync::mpsc::Sender<Message>, tokio::sync::mpsc::Receiver<Message>) {
        mpsc::channel::<Message>(super::SIDS_DEFAULT_BUFFER_SIZE)
    }

    fn create_blocking_actor_channel(&self) -> (std::sync::mpsc::Sender<Message>, std::sync::mpsc::Receiver<Message>) {
        std::sync::mpsc::channel::<Message>()
    }

    fn create_response_channel(&self) -> (tokio::sync::oneshot::Sender<ResponseMessage>, tokio::sync::oneshot::Receiver<ResponseMessage>) {
        oneshot::channel::<ResponseMessage>()
    }

    fn create_blocking_response_channel(&self) -> (std::sync::mpsc::Sender<ResponseMessage>, std::sync::mpsc::Receiver<ResponseMessage>) {
        std::sync::mpsc::channel::<ResponseMessage>()
    }
    
    fn create_typed_actor_channel<T>(&self) -> (mpsc::Sender<MessageImpl<T>>, mpsc::Receiver<MessageImpl<T>>) {
        mpsc::channel::<MessageImpl<T>>(SIDS_DEFAULT_BUFFER_SIZE)
    }
    
    fn create_typed_response_channel<T>(&self) -> (oneshot::Sender<ResponseMessageImpl<T>>, oneshot::Receiver<ResponseMessageImpl<T>>) {
        oneshot::channel::<ResponseMessageImpl<T>>()
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
        let msg = GuardianMessage::Terminate;
         self.guardian_ref.send(msg).await;
    }

    pub (super) async fn count_officers(&mut self) -> Result<u32, Error> {
        let (tx, rx) = self.create_response_channel();
        let msg = GuardianMessage::CountOfficers { responder: tx };
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::ResponseCount(_message, count)) => Ok(count),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to get officer count")), // grcov-excl-line
        }
    }

    pub (super) async fn count_courriers(&mut self, officer_id: u32) -> Result<u32, Error> {
        let (tx, rx) = self.create_response_channel();
        let msg = GuardianMessage::CountCourriers { officer_id, responder: tx };
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::ResponseCount( _message,  count )) => Ok(count),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to get courrier count")), // grcov-excl-line
        }
    }

    pub (super) async fn count_actors(&mut self) -> Result<u32, Error> {
        let (tx, rx) = self.create_response_channel();
        let msg = GuardianMessage::CountActors { responder: tx };
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::ResponseCount (_message, count )) => Ok(count),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to get actor count")), // grcov-excl-line
        }
    }


    pub (super) async fn create_officer<T: Actor + 'static>(&mut self, name: Option<String>, actor_type: T) -> Result<(), Error> {
        info!("Creating officer called {}", name.clone().unwrap_or("Unnamed".to_string()));
        // Need to figure out how to get the actor to the officer. Should we create the actor ref here?)
        let (tx, _rx) = self.create_response_channel();
        let (tx2, rx2) = self.create_actor_channel();
        let actor = ActorImpl::new(name, actor_type, rx2);
        let actor_ref = ActorRef::new(actor, tx2);
        let msg = GuardianMessage::CreateOfficer { officer_type: actor_ref, responder: tx  }; // TODO: change this to the correct message for creating an officer
        self.guardian_ref.send(msg).await;
        match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to create officer")), // grcov-excl-line
        }
    }

    pub (super) async fn create_blocking_officer<T: Actor + 'static>(&mut self, actor_type: T, name: Option<String>) -> Result<(), Error> {
        info!("Creating blocking officer called {}", name.clone().unwrap_or("Unnamed".to_string()));
        let (tx2, rx2) = self.create_blocking_actor_channel();
        // create actor reference here and send it to the guardian.
        let (tx, rx) = self.create_blocking_response_channel();
        let actor = BlockingActorImpl::new(name, actor_type, rx2);
        let actor_ref = BlockingActorRef::new(actor, tx2);
        // send message to guardian to create officer
        let msg = GuardianMessage::CreateBlockingOfficer { officer_type: actor_ref, responder: tx };
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
        let (tx, _rx) = self.create_response_channel();
        let msg = GuardianMessage::RemoveOfficer { officer_id, responder: tx };
        self.guardian_ref.send(msg).await;
       let _ = match _rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove officer")),
        };
        Ok(())
    }

    pub async fn dispatch<T>(&mut self, officer_id : u32, message:MessageImpl<T>, blocking: bool) {
        info!("Dispatching message to actor system");
        self.guardian_ref.send(GuardianMessage::OfficerMessage { officer_id, message, blocking }).await;        
    }

    pub async fn add_courrier<T: Actor + 'static>(&mut self, officer_id: u32, courrier_type: T, name: Option<String>, blocking: bool) -> Result<(), Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let (tx2, rx2) = tokio::sync::mpsc::channel::<Message>(super::SIDS_DEFAULT_BUFFER_SIZE);
        let actor = ActorImpl::new(name, courrier_type, rx2);
        let actor_ref = ActorRef::new(actor, tx2);
        let msg = GuardianMessage::AddCourrier { officer_id , courrier_type: actor_ref, responder: tx, blocking }; // TODO: convert to correct message type when available
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to add courrier")), // grcov-excl-line
        }
    }

    pub async fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32, blocking: bool) -> Result<(), Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::RemoveCourrier { officer_id, courrier_id, responder: tx, blocking }; // TODO: convert to correct message type when available
        self.guardian_ref.send(msg).await;
        match rx.await {
            Ok(ResponseMessage::Success) => Ok(()),
            Ok(ResponseMessage::Failure) => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")),
            _ => Err(Error::new(ErrorKind::InvalidInput, "Failed to remove courrier")), // grcov-excl-line
        }
    }

    pub async fn notify_courriers(&mut self, officer_id: u32, message:Message, blocking: bool) -> Result<(), Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let msg = GuardianMessage::NotifyCourriers { officer_id, message, responder: tx, blocking };
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