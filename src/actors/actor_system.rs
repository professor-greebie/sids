use super::{actor_ref::TokioActorRef, guardian::Guardian, messages::Message};
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
    guardian : TokioActorRef,
}


impl ActorSystem {
    /// Create a new ActorSystem
    /// 
    /// The ActorSystem will start by launching a guardian, which is a non-blocking officer-actor that manages all other actors in the system.
    /// The guardian will be dormant until start_system is called in the ActorSystem.
    pub fn new() -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(super::SIDS_DEFAULT_BUFFER_SIZE);
        let actor = Guardian::new(rec);
        let actor_ref = TokioActorRef::new(super::actor::Actor::Guardian(actor), snd);

        ActorSystem {
            guardian: actor_ref,
        }
    }

    pub async fn start_system(&mut self) {

        //TODO: Ready the guardian actor with a message to start the system.
        //self.guardian.run().await;
    }

    pub async fn dispatch(&mut self, message: Message) {
        info!("Dispatching message to actor system");
        self.guardian.send(&message).await;
    }

    

}




#[cfg(test)]
mod tests {
    use crate::actors::messages;

    use super::*;
    use tokio::sync::oneshot;

    #[tokio::test]
    #[allow(dead_code)]
    async fn test_actor_system_is_not_headless_when_started() {
        let mut actor_system = ActorSystem::new();
        let (tx, _rx) = oneshot::channel();
        // message responder is never used in the message. Should we change the message to not include the responder?
        let _message = messages::Message::ActorMessage(messages::ActorMessage::GetNextId { responder: tx });
        actor_system.start_system().await;
        actor_system.dispatch(_message).await;
        assert!(true);


        //actor_system.dispatch(&message).await;
        //let response = rx.await.unwrap();
        //assert_eq!(response, 1);
        
    }
}