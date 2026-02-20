extern crate sids;

use env_logger::{Builder, Env};
use log::info;
use sids::actors::actor_ref::ActorRef;
use sids::actors::{get_response_channel, send_message_by_id, spawn_actor, start_actor_system};
use std::collections::HashMap;
use sids::actors::actor::{Actor, ActorImpl};
use sids::actors::messages::{Message, ResponseMessage};

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

// Sample MType for the chat system. 
// Once a message is selected, all actors in the system must use the same message type.

#[derive(Debug, Clone)]
enum ChatMessage {
    Hello { name: String },
    Goodbye,
    StringMessage(String),
}


/// Alice and Bob will send messages to each other.
struct Alice {
    partners: HashMap<String, ActorRef<ChatMessage, ResponseMessage>>,
}
impl Alice {
    fn new() -> Self {
        Alice { partners: HashMap::new() }
    }
    // Actor needs to be static to ensure that the actor is not moved before the message is sent.
    fn add_partner<T: Actor<ChatMessage, ResponseMessage> + 'static>(&mut self, partner: T, name: String, thread_ref: &'static std::sync::atomic::AtomicUsize, message_ref: &'static std::sync::atomic::AtomicUsize) {
        let (sender, receiver) = tokio::sync::mpsc::channel::<Message<ChatMessage, ResponseMessage>>(100);
        let actor = ActorImpl::<T, ChatMessage, ResponseMessage>::new(Some(name.clone()), partner, receiver);
        let reference = ActorRef::new(actor, sender, thread_ref, message_ref);
        self.partners.insert(name, reference);
    }

}
    
impl Actor<ChatMessage, ResponseMessage> for Alice {
    async fn receive(&mut self,message:Message<ChatMessage, ResponseMessage>) where Self:Sized+'static {
        match message {
            Message {payload: Some(ChatMessage::Hello { name: name_string}), stop: _, responder: _, blocking: _} => {
                info!("Alice received a Hello message");
                let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
                let reference = self.partners.get_mut(&name_string).unwrap();
                info!("Alice is sending a message to Bob");
                let _ = reference.send(Message {payload: Some(ChatMessage::Hello{name: name_string}), stop: false, responder: Some(tx), blocking: None}).await;
                info!("Alice sent a message to Bob. Awaiting response.");
                let response = rx.await.expect("Failed to receive response");
                info!("Alice received a response: {:?}", response);
            },
            Message {payload: Some( ChatMessage::Goodbye), stop: _, responder: _, blocking: _} => {
                info!("Alice received a Goodbye message");
            },
            Message {payload: Some(ChatMessage::StringMessage(message)), stop: _, responder: Some(response), blocking: _} => {
                // IMPORTANT: This must come from a responder and not the sender to avoid race conditions.
                info!("Alice received a message: {}", message);
                let _ = response.send(ResponseMessage::Complete);

        },
        _ => {
            info!("Alice received a message with no information.");
        }
    }
}
}
    
// Bob is a simple actor that only responds to messages and does not send any messages himself.
struct Bob;
impl Bob {
    fn new() -> Self {
        Bob
    }
}
impl Actor<ChatMessage, ResponseMessage> for Bob {
    async fn receive(&mut self,message:Message<ChatMessage, ResponseMessage>) where Self:Sized+'static {
        match message {
            Message {payload: Some(ChatMessage::Hello { name: name_string}), stop: _, responder: Some(courrier), blocking: _} => {
                info!("{} received a Hello message", name_string);
                let _ = courrier.send(ResponseMessage::Success);
            },
            Message {payload: Some(ChatMessage::StringMessage(message)), stop: _, responder: Some(courrier), blocking: _} => {
                info!("Bob received a message: {}", message);
                let _ = courrier.send(ResponseMessage::Complete);
        },
        _ => {
            info!("Bob received a message with irrelevant information.");
        }
    }

}}

/// implement the actor system and send some messages between Alice and Bob.
async fn start_sample_actor_system() {
    let mut actor_system = start_actor_system::<ChatMessage, ResponseMessage>();
    let thread_ref = actor_system.get_thread_count_reference();
    let message_ref = actor_system.get_message_count_reference();
    let bob = Bob::new();
    let mut alice = Alice::new();
    alice.add_partner(bob, "Bob".to_string(), thread_ref, message_ref);
    let (tx, rx) = get_response_channel(&actor_system);
    spawn_actor(&mut actor_system, alice, Some("Alice".to_string())).await;
    let hello = Message {
        payload: Some(ChatMessage::Hello { name: "Bob".to_string() }),
        stop: false,
        responder: None,
        blocking: None,
    };
    let goodbye = Message {
        payload: Some(ChatMessage::Goodbye),
        stop: false,
        responder: None,
        blocking: None,
    };
    let string_message = Message {
        payload: Some(ChatMessage::StringMessage("Hello Alice".to_string())),
        stop: false,
        responder: Some(tx),
        blocking: None,
    };
    send_message_by_id(&mut actor_system, 0, hello).await;
    send_message_by_id(&mut actor_system, 0, goodbye).await;
    send_message_by_id(&mut actor_system, 0, string_message).await;
    let response = rx.await.expect("Failed to receive response");
    info!("We received a response: {:?} from Alice", response);
    info!("Total messages sent: {}", actor_system.get_message_count());
    info!("Total threads: {}", actor_system.get_thread_count());


}
#[tokio::main]
async fn main() {
    get_loggings();
    start_sample_actor_system().await;
}