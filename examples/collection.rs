extern crate sids;

use env_logger::{Builder, Env};
use log::info;
use sids::actors::community::collector::Collector;
use sids::actors::messages::Message;

// NOTE: This will collect some data from the internet and store it in the current directory.

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    user_id: u32,
    id: u32,
    title: String,
    body: String,
}






async fn start_sample_actor_system() {
    let collector1 = Collector;
    let collector2 = Collector;
    let collector3 = Collector;
    let serde_actor1 = SerdeActor::<Post>;  // actor to handle the deserialization of the json
    let json_url = "https://jsonplaceholder.typicode.com/posts";
    let mut actor_system = sids::actors::api::start_actor_system();
    sids::actors::api::spawn_blocking_officer(&mut actor_system, Some("Collector 1".to_string()), collector1).await;
    sids::actors::api::spawn_blocking_officer(&mut actor_system, Some("Collector 2".to_string()), collector2).await;
    sids::actors::api::spawn_blocking_officer(&mut actor_system, Some("Collector 3".to_string()), collector3).await;

    info!("Sending messages to officers");
    let (tx1, rx1) = tokio::sync::oneshot::channel();
    let (tx2, rx2) = tokio::sync::oneshot::channel();
    let (tx3, rx3) = tokio::sync::oneshot::channel();


    let message1 = Message::GetUrl { url: json_url.to_string(), output: "./rust.html_sample".to_string(), responder: tx1 };
    let message2 = Message::GetUrl { url: "https://www.google.com".to_string(), output: "./google.html_sample".to_string(), responder: tx2 };
    let message3 = Message::GetUrl { url: "https://www.github.com".to_string(), output: "./github.html_sample".to_string(), responder: tx3 };
    sids::actors::api::send_message_to_officer_enum(&mut actor_system, 0, message1, true).await;
    sids::actors::api::send_message_to_officer_enum(&mut actor_system, 1, message2, true).await;
    sids::actors::api::send_message_to_officer_enum(&mut actor_system, 2, message3, true).await;

    rx1.await.unwrap();
    rx2.await.unwrap();
    rx3.await.unwrap();

    info!("Finished sending messages to officers");
    //sids::actors::api::terminate_actor_system(&mut actor_system).await;
    

}

#[tokio::main]
async fn main() {
    get_loggings();
    start_sample_actor_system().await;
}
