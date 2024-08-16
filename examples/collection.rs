extern crate sids;

use env_logger::{Builder, Env};

// NOTE: This will collect some data from the internet and store it in the current directory.

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

async fn start_sample_actor_system() {
    // Start an actor system and spawn a few officers with some actors that collect stuff from a url

}

#[tokio::main]
async fn main() {
    get_loggings();
    start_sample_actor_system().await;
}
