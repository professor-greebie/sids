use std::io::Error;

use log::info;
use sids::actors::{actor::SelectActor, actor_system::ActorSystem, messages::KafkaProducerMessage};
use env_logger::{Builder, Env};


fn init_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    init_logger();

    let mut _actor_system = ActorSystem::new();
    //_actor_system.spawn_actor(SelectActor::GetActor);
    _actor_system.spawn_actor(SelectActor::KafkaProducer);
    let _next_actor_system =_actor_system.next_actor().unwrap()._value.as_ref().unwrap();
    info!("Sending message to get actor reference");
    let _ = _next_actor_system.clone().send(KafkaProducerMessage::Produce{
        topic: "test".to_string(),
        message: "Hello".to_string(),
        responder: tokio::sync::oneshot::channel(),
    });

    Ok(())

    
}


