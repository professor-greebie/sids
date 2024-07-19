use super::guardian::Guardian;


pub struct ActorSystem {
    
    guardian : Guardian,
}


impl ActorSystem {
    pub fn new() -> Self {
        let (snd, rec) = tokio::sync::mpsc::channel(4);
        let actor = Guardian::new(rec);
        ActorSystem {
            guardian: actor,
        }
    }

    pub async fn start_system(&mut self) {
        self.guardian.run().await;
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


        //actor_system.dispatch(&message).await;
        //let response = rx.await.unwrap();
        //assert_eq!(response, 1);
        
    }
}