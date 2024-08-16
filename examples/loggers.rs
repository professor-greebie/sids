extern crate sids;
use env_logger::{Builder, Env};


// Idea here is that using the logs, we can provide an animation of actors receiving messages and sending messages to each other.

fn get_loggings() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

async fn start_sample_actor_system() {
    // Start an actor system and spawn a few officers with some actors that send messages to each other.
    
}

#[tokio::main]
async fn main () {
    get_loggings();
    start_sample_actor_system().await;
}