use std::io::Error;

use log::info;
use sids::actors::{actor::SelectActor, actor_system::ActorSystem};
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
    _actor_system.spawn_actor(SelectActor::GetActor);
    let _next_actor_system =_actor_system.next_actor().unwrap()._value.as_ref().unwrap();
    info!("Sending message to get actor reference");
    let _ = _next_actor_system.clone().get_uri_result("https://www.rust-lang.org".to_string(), "./test.html".to_string());

    Ok(())

    
}


