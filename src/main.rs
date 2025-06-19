use env_logger::{Builder, Env};
use log::info;
use std::io::Error;

pub mod actors;

fn init_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_logger();
    info!("Sending message to get actor reference");
    Ok(())
}


