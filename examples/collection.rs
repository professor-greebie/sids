extern crate sids;

use env_logger::{Builder, Env};
use sids::actors::api::*;
use sids::actors::messages::Message;
use sids::actors::officer::SelectActor;

// NOTE: This will collect some data from the internet and store it in the current directory.

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

async fn start_sample_actor_system() {
    let mut _actor_system = start_actor_system();
    //_actor_system.spawn_actor(SelectActor::GetActor);
    spawn_officer(&mut _actor_system, SelectActor::Collector).await;
    spawn_officer(&mut _actor_system, SelectActor::Collector).await;
    spawn_officer(&mut _actor_system, SelectActor::Collector).await;

    for j in 0..3 {
        std::thread::sleep(std::time::Duration::from_secs(4));
        send_message_to_officer(
            &mut _actor_system,
            j,
            Message::GetURI {
                uri: "http://www.example.com".to_string(),
                location: "./tmp_example".to_string(),
            },
        )
        .await;
    }
}

#[tokio::main]
async fn main() {
    get_loggings();
    start_sample_actor_system().await;
}
