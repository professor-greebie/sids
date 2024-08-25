extern crate sids;
use env_logger::{Builder, Env};
use log::info;
use sids::actors::actor::ActorTrait;
use sids::actors::api::*;
use sids::actors::messages::InternalMessage;

// Idea here is that using the logs, we can provide an animation of actors receiving messages and sending messages to each other.

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

// Sample actor type that will be used to send messages to each other.
#[derive(Debug)]
struct SampleActor {
    name: String,
}

impl SampleActor {
    fn new(name: String) -> Self {
        SampleActor { name }
    }
}

impl ActorTrait for SampleActor
where
    SampleActor: 'static,
{
    fn receive(&mut self, message: InternalMessage) {
        let name = self.name.clone();
        // Log the message received by the actor.
        info!("Actor {} received message: {:?}", self.name, message);
        if let InternalMessage::StringMessage { message } = message {
            info!("Actor {} received string message: {}", name, message);
        }
    }
}

struct SampleBlockingActor;

impl ActorTrait for SampleBlockingActor
where
    SampleBlockingActor: 'static,
{
    fn receive(&mut self, message: InternalMessage) {
        // do nothing
        info!("Received message in blocking actor via ActorTrait");
        if let InternalMessage::StringMessage { message } = message {
            info!("Blocking actor received string message: {}", message);
        }
    }
}



async fn start_sample_actor_system() {
    // Start an actor system and spawn a few officers with some actors that send messages to each other.
    let mut actor_system = sids::actors::api::start_actor_system();
    let actor_type = SampleActor::new("Actor 1".to_string());
    let actor_type2 = SampleActor::new("Actor 2".to_string());
    let actor_type3 = SampleActor::new("Actor 3".to_string());
    spawn_officer(&mut actor_system, Some(actor_type.name.clone()), actor_type).await;
    spawn_officer(&mut actor_system, None, actor_type2).await;
    spawn_blocking_officer(
        &mut actor_system,
        Some("Generic blocking actor".to_string()),
        SampleBlockingActor,
    )
    .await;
    spawn_officer(
        &mut actor_system,
        Some(actor_type3.name.clone()),
        actor_type3,
    )
    .await;

    let message = "Hello Actor 1".to_string();
    // Send messages to the actors.
    send_message_to_officer(&mut actor_system, 0, message, false).await;
    send_message_to_officer(&mut actor_system, 1, "what's up?".to_string(), false).await;
    send_message_to_officer(
        &mut actor_system,
        0,
        "I am a blocking actor".to_string(),
        true,
    )
    .await;
    send_message_to_officer_enum(&mut actor_system, 0, InternalMessage::Terminate, false).await;
}

#[tokio::main]
async fn main() {
    get_loggings();
    start_sample_actor_system().await;
}
