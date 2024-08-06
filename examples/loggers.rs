extern crate sids;
use env_logger::{Builder, Env};
use sids::actors::api::*;
use sids::actors::officer::SelectActor;
use sids::actors::messages::Message;

// Idea here is that using the logs, we can provide an animation of actors receiving messages and sending messages to each other.

fn get_loggings() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

async fn start_sample_actor_system() {
    let mut _actor_system = start_actor_system();
    //_actor_system.spawn_actor(SelectActor::GetActor);
    spawn_officer(&mut _actor_system, SelectActor::LogActor).await;
    spawn_officer(&mut _actor_system, SelectActor::LogActor).await;
    spawn_officer(&mut _actor_system, SelectActor::LogActor).await;
    for i in 0..30 {
        let actor = i % 3;
        std::thread::sleep(std::time::Duration::from_secs(4));
        send_message_to_officer(&mut _actor_system, actor, Message::LogMessage { message: "hello".to_string() }).await;
    }
    
}

#[tokio::main]
async fn main () {
    get_loggings();
    start_sample_actor_system().await;
}