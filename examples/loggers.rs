extern crate sids;

use env_logger::{Builder, Env};
use log::info;
use sids::actors::{ping_actor_system, send_message_by_id, spawn_actor, spawn_blocking_actor};
use std::thread;
use sids::actors::actor::Actor;
use sids::actors::messages::{Message, ResponseMessage};

// Idea here is that using the logs, we can provide an animation of actors receiving messages and sending messages to each other.

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

// Sample actor type that will be used to send messages to each other.
#[derive(Debug)]

enum GenericMessage {
    Hello, 
    Goodbye
}
#[derive(Debug)]
struct SampleActor {
    name: String,
}

impl SampleActor {
    fn new(name: String) -> Self {
        SampleActor { name }
    }
}

impl Actor<GenericMessage> for SampleActor
where
    SampleActor: 'static,
{
    fn receive(&mut self, message: Message<GenericMessage>) {
        let name = self.name.clone();
        // Log the message received by the actor.
        info!("Actor {} received message: {:?} on thread #: {:?}", name, message.payload, thread::current().id());
    }
}

struct SampleBlockingActor;

impl Actor<GenericMessage> for SampleBlockingActor
where
    SampleBlockingActor: 'static,
{
    fn receive(&mut self, message: Message<GenericMessage>) {
        // do nothing
        info!("Received message in blocking actor via Actor trait");
        info!("Received message in blocking actor: {:?} on thread #: {:?} ", message, thread::current().id());
        if let Message {payload: _, stop: false, responder: _, blocking: Some(block)} =  &message {

            block.send(ResponseMessage::SUCCESS).unwrap();
            
        }
    
    }
}



async fn start_sample_actor_system() {
    
    // Start an actor system and spawn a few officers with some actors that send messages to each other.
    let mut actor_system = sids::actors::start_actor_system::<GenericMessage>();
    let (tx, rx) = sids::actors::get_response_channel(&mut actor_system);
    let (btx, brx) = sids::actors::get_blocking_response_channel(&mut actor_system);
    let actor_type = SampleActor::new("Actor 1".to_string());
    let actor_type2 = SampleActor::new("Actor 2".to_string());
    let actor_type3 = SampleActor::new("Actor 3".to_string());
    let actor_type4 = SampleBlockingActor;
    spawn_actor(&mut actor_system, actor_type, Some("Actor 1".to_string())).await;
    spawn_actor(&mut actor_system, actor_type2, Some("Actor 2".to_string())).await;
    spawn_actor(&mut actor_system, actor_type3, Some("Actor 3".to_string())).await;
    spawn_blocking_actor(&mut actor_system, actor_type4, Some("Blocking Actor".to_string())).await;
    // Send a message to the first actor.
    let message = Message {
        payload: Some(GenericMessage::Hello),
        stop: false,
        responder: Some(tx),
        blocking: None,
    };
    let message2 = Message {
        payload: Some(GenericMessage::Goodbye),
        stop: false,
        responder: None,
        blocking: None,
    };
    let message3 = Message {
        payload: Some(GenericMessage::Hello),
        stop: false,
        responder: None,
        blocking: None,
    };
    let message4 = Message {
        payload: Some(GenericMessage::Hello),
        stop: false,
        responder: None,
        blocking: Some(btx),
    };
    ping_actor_system(&mut actor_system).await;
    send_message_by_id(&mut actor_system, 0, message).await;
    send_message_by_id(&mut actor_system, 1, message2).await;
    send_message_by_id(&mut actor_system, 2, message3).await;
    send_message_by_id(&mut actor_system, 0, message4).await;
    if let Ok(response) = rx.await {
        info!("Response received from guardian: {:?}", response);
    }  
    if let Ok(response) = brx.recv() {
        info!("Response received from blocking actor {:?}", response);
    } 
}

#[tokio::main]
async fn main() {
    get_loggings();
    start_sample_actor_system().await;
}
