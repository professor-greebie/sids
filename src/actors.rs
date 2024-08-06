pub mod actor;
pub mod actor_ref;
pub mod actor_system;
pub mod guardian;
pub mod messages;
pub mod officer;

static SIDS_DEFAULT_BUFFER_SIZE: usize = 100;

pub mod api {

    use crate::actors::actor_system::ActorSystem;

    use super::officer::SelectActor;

    pub fn start_actor_system() -> ActorSystem {
        let actor_system = ActorSystem::new();
        actor_system
    }

    pub async fn spawn_officer(
        actor_system: &mut ActorSystem,
        actor_type: SelectActor,
    ) -> &mut ActorSystem {
        let _ = actor_system
            .create_officer(actor_type)
            .await
            .expect("Failed to create officer");
        actor_system
    }

    pub async fn spawn_courrier(
        actor_system: &mut ActorSystem,
        officer_id: u32,
        courrier_type: SelectActor,
    ) -> &mut ActorSystem {
        let _ = actor_system
            .add_courrier(officer_id, courrier_type)
            .await
            .expect("Failed to add courrier");
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
        message: crate::actors::messages::Message,
    ) -> &mut ActorSystem {
        actor_system
            .dispatch(officer_id, message)
            .await;
        actor_system
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_actor_system() {
            let mut _actor_system = start_actor_system();
            spawn_officer(&mut _actor_system, SelectActor::LogActor).await;
            spawn_courrier(&mut _actor_system, 0, SelectActor::LogActor).await;
        }
    }
}
