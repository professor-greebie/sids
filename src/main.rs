use std::io::Error;

use sids::actors::actor_system::ActorSystem;


#[tokio::main]
async fn main() -> Result<(), Error> {

    let actor_system = ActorSystem::new();

    Ok(())

    
}
