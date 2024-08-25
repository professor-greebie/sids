pub mod actor;
pub mod actor_ref;
pub mod actor_system;
pub mod guardian;
pub mod messages;
pub mod officer;
pub mod community;

static SIDS_DEFAULT_BUFFER_SIZE: usize = 100;

pub mod api {
    use log::info;

    use super::messages::Message;

    use crate::actors::actor_system::ActorSystem;

    use super::actor::Actor;

    pub fn start_actor_system() -> ActorSystem {
        ActorSystem::new()
    }

    pub async fn spawn_officer<T: Actor + 'static>(
        actor_system: &mut ActorSystem,
        name: Option<String>, 
        actor_type: T,
    ) -> &mut ActorSystem {
        info!("Spawning officer");
        actor_system
            .create_officer(name, actor_type)
            .await
            .expect("Failed to create officer");
        actor_system
    }

    pub async fn spawn_blocking_officer<T: Actor + 'static>(
        actor_system: &mut ActorSystem,
        name: Option<String>, 
        actor_type: T,
    ) -> &mut ActorSystem {
        info!("Spawning blocking officer");
        actor_system
            .create_blocking_officer(actor_type, name).await
            .expect("Failed to create blocking officer");
        actor_system
    }

    pub async fn spawn_courrier<T: Actor + 'static>(
        actor_system: &mut ActorSystem,
        officer_id: u32,
        courrier_type: T,
        name: Option<String>, 
        blocking: bool
    ) -> &mut ActorSystem {
        actor_system
            .add_courrier(officer_id, courrier_type, name, blocking)
            .await
            .expect("Failed to add courrier");
        actor_system
    }

    pub async fn remove_officer(actor_system: &mut ActorSystem, officer_id: u32) -> &mut ActorSystem {
        actor_system
            .remove_officer(officer_id)
            .await
            .expect("Failed to remove officer");
        actor_system
    }

    pub async fn remove_courrier(
        actor_system: &mut ActorSystem,
        officer_id: u32,
        courrier_id: u32,
        blocking: bool
    ) -> &mut ActorSystem {
        actor_system
            .remove_courrier(officer_id, courrier_id, blocking)
            .await
            .expect("Failed to remove courrier");
        actor_system
    }

    pub async fn terminate_actor_system(actor_system: &mut ActorSystem) -> &mut ActorSystem {
        actor_system
            .stop_system()
            .await;
        actor_system
    }

    pub async fn send_message_to_officer (
        actor_system: &mut ActorSystem,
        officer_id: u32,
        message: String,
        blocking: bool
    ) -> &mut ActorSystem {
        let message = Message::StringMessage { message};
        actor_system
            .dispatch(officer_id, message, blocking)
            .await;
        actor_system
    }

    pub async fn send_message_to_officer_enum (
        actor_system: &mut ActorSystem,
        officer_id: u32,
        message: Message,
        blocking: bool
    ) -> &mut ActorSystem {
        actor_system
            .dispatch(officer_id, message, blocking)
            .await;
        actor_system
    }

    #[cfg(test)]
    mod tests {

        use crate::actors::{actor::Actor, messages::Message};

        use super::*;

        struct Logging;
        impl Actor for Logging {
            fn receive(&mut self, message: Message) {
                log::info!("Logging message: {:?}", message);
            }

        }

        #[tokio::test]
        async fn test_actor_system() {
            // Create an actor.S
            let logging = Logging;
            let mut _actor_system = start_actor_system();
            spawn_officer(&mut _actor_system, None, logging).await;
            //spawn_courrier(&mut _actor_system, 0,logging).await;
        }
    }
}
