use std::io::Error;


use log::info;
use env_logger::{Builder, Env};

pub mod actors;



fn init_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    init_logger();

    //let _actor_system = start_actor_system();
    //let collector = spawn_collector(_actor_system).to_owned();
    //spawn_kafka_producer_actor(collector);
    //let _next_actor_system = next_actor_system(_actor_system);
    //describe_actor_system(&_actor_system);
    //send_message(_next_actor_system, Message::KafkaProducerMessage(KafkaProducerMessage::Produce{topic: "junk".to_string(), message: "Is this working?".to_string() })).await;
    info!("Sending message to get actor reference");

    Ok(())
}


