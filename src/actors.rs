pub mod actor;
pub mod officer;
pub mod actor_ref;
pub mod actor_system;
pub mod messages;
pub mod guardian;


static SIDS_DEFAULT_BUFFER_SIZE: usize = 100;

mod emmissary_system {

    use crate::actors::actor_system::ActorSystem;

    use super::messages;

    #[allow(dead_code)]
    pub fn start_actor_system() -> ActorSystem {
        let actor_system = ActorSystem::new();
        actor_system
    }
    #[allow(dead_code)]
    pub fn spawn_actor(actor_system: &mut ActorSystem) -> &ActorSystem {

        // This is a temporary solution to spawn actors. It should be replaced with a more robust solution.
        actor_system
    }
    #[allow(dead_code)]
    pub fn spawn_collector(actor_system: &mut ActorSystem) -> &ActorSystem {
        // This is a temporary solution to spawn actors. It should be replaced with a more robust solution.
        actor_system
    }
    #[allow(dead_code)]
    pub fn spawn_kafka_producer_actor(actor_system:  &mut ActorSystem) -> &ActorSystem {
        // This is a temporary solution to spawn actors. It should be replaced with a more robust solution.
        actor_system
    }
    #[allow(dead_code)]
    pub fn next_actor_system(actor_system: &mut ActorSystem) -> &ActorSystem {
        actor_system
    }
    #[allow(dead_code)]
    pub async fn send_message(_actor_system: &ActorSystem, _message: messages::InternalMessage) {
    
    }
    #[allow(dead_code)]
    pub fn send_get_request(_actor_system: &ActorSystem, _uri: String, _location: String) {
    
    }
    #[allow(dead_code)]
    pub fn describe_actor_system(_actor_system: &ActorSystem) {
        log::info!(actor = "Actor System"; "hello");
    
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_actor_system() {
            let _actor_system = start_actor_system();

        }
    }
}
