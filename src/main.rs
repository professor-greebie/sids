use std::io::Error;

use actors::actor::{next_actor_system, KafkaProducerMessage};
use log::info;
use env_logger::{Builder, Env};

pub mod actors;
use crate::actors::actor::{start_actor_system, spawn_kafka_producer_actor, send_message, Message};


fn init_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    init_logger();

    let mut _actor_system = start_actor_system();
    //_actor_system.spawn_actor(SelectActor::GetActor);
    spawn_kafka_producer_actor(&mut _actor_system);
    let _next_actor_system = next_actor_system(_actor_system);
    send_message(_next_actor_system, Message::KafkaProducerMessage(KafkaProducerMessage::Produce{topic: "junk".to_string(), message: "Hello".to_string() })).await;
    info!("Sending message to get actor reference");

    Ok(())
}


