use log::info;
use std::collections::HashMap;
use std::io::Error;
use std::ops::DerefMut;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use super::actor::Actor;
use super::actor_ref::ActorRef;
use super::messages::{Message, ResponseMessage};



#[derive(Debug)]
pub (super) enum GuardianAction {
    CREATE,
    SEND,
    RECEIVE,
    DELETE,
}

#[derive(Debug)]
pub (super) struct GuardianMessage<MType>{
    pub (super) action: GuardianAction,
    pub (super) actor_id: u32,
    pub (super) message: Message<MType>,
    pub (super) responder: tokio::sync::oneshot::Sender<ResponseMessage>,   
}


#[derive(Debug)]
pub(super) struct Guardian<MType: Send + 'static> {
    pub (super) active_message: Option<Message<GuardianMessage<MType>>>,
    pub(super) actors: HashMap<u32, ActorRef<MType>>,
}

impl <MType: Send + 'static> Guardian<MType> {
    pub(super) fn new() -> Guardian<MType> {
        Guardian::<MType> { 
            active_message : None,
            actors: HashMap::<u32, ActorRef<MType>>::new(),
        }
    }

    pub (super) fn create_actor(&mut self, actor: ActorRef<MType>) -> Result<(), Error> {
        let id = self.actors.len() as u32;
        self.actors.insert(id, actor);
        info!("Actor created with id: {}", id);
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
    impl Actor<String> for SampleActor {
        fn receive(&mut self, _message: Message<String>) {
            // do nothing
            info!("Received message");
            let Message { payload } = _message;
            info!("Received message: {}", payload);
        }
    }

    #[tokio::test]
    async fn test_guardian_actor() {
        let (actor_tx, actor_rx) = mpsc::channel(1);
        let mut guardian = Guardian::<String>::new();
        let actor = ActorImpl::new(None, SampleActor, actor_rx);
        let officer = ActorRef::new(actor, actor_tx);
        let result = guardian.create_actor(officer);
        assert!(result.is_ok());
        let (rec_tx, rec_rx) = oneshot::channel::<ResponseMessage>();
        let _ = guardian
            .receive(Message { payload: GuardianMessage {
                actor_id: 0,
                message: Message { payload: "Hello".to_string() },
                responder: rec_tx,
            }});
        let response = rec_rx.await.unwrap();
        assert_eq!(response, ResponseMessage::SUCCESS);
    }

    #[tokio::test]
    async fn test_guardian_actor_count_actors() {
        
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
