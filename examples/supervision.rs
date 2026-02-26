#[cfg(feature = "visualize")]
use sids::actors::actor::Actor;
#[cfg(feature = "visualize")]
use sids::actors::messages::Message;
#[cfg(feature = "visualize")]
use sids::actors::response_handler::from_oneshot;
/// Example demonstrating the supervision system for monitoring and visualizing actors
///
/// This example shows how to:
/// 1. Create actors and spawn them
/// 2. Track actor metrics automatically
/// 3. Export supervision data in multiple formats (JSON, DOT, Mermaid)
/// 4. Generate visualizations of the actor system
#[cfg(feature = "visualize")]
use sids::actors::{
    messages::ResponseMessage, send_message_by_id, spawn_actor, start_actor_system_with_config,
};
#[cfg(feature = "visualize")]
use sids::config::SidsConfig;
#[cfg(feature = "visualize")]
use sids::supervision_export::{to_dot, to_json, to_mermaid, to_mermaid_sequence, to_text_summary};

#[cfg(feature = "visualize")]
struct RequestActor;

#[cfg(feature = "visualize")]
impl Actor<String, ResponseMessage> for RequestActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        if let Some(payload) = message.payload {
            println!("RequestActor received: {}", payload);
        }
        if let Some(responder) = message.responder {
            responder.handle(ResponseMessage::Success).await;
        }
    }
}

#[cfg(feature = "visualize")]
struct WorkerActor;

#[cfg(feature = "visualize")]
impl Actor<String, ResponseMessage> for WorkerActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        if let Some(payload) = message.payload {
            println!("WorkerActor processing: {}", payload);
        }
        if let Some(responder) = message.responder {
            responder.handle(ResponseMessage::Success).await;
        }
    }
}

#[cfg(feature = "visualize")]
struct CoordinatorActor;

#[cfg(feature = "visualize")]
impl Actor<String, ResponseMessage> for CoordinatorActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        if let Some(payload) = message.payload {
            println!("CoordinatorActor coordinating: {}", payload);
        }
        if let Some(responder) = message.responder {
            responder.handle(ResponseMessage::Success).await;
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(not(feature = "visualize"))]
    {
        println!("This example requires the 'visualize' feature.");
        println!("Run with: cargo run --example supervision --features visualize");
        return;
    }

    #[cfg(feature = "visualize")]
    {
        if let Err(e) = run_supervision_demo().await {
            eprintln!("Supervision demo failed: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "visualize")]
async fn run_supervision_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SIDS Supervision System Example ===\n");

    // Create actor system with default config
    let config = SidsConfig::default();
    let mut actor_system = start_actor_system_with_config::<String, ResponseMessage>(config);

    // Spawn some actors
    println!("Spawning actors...");
    spawn_actor(
        &mut actor_system,
        RequestActor,
        Some("RequestActor".to_string()),
    )
    .await;

    spawn_actor(
        &mut actor_system,
        WorkerActor,
        Some("WorkerActor".to_string()),
    )
    .await;

    spawn_actor(
        &mut actor_system,
        CoordinatorActor,
        Some("CoordinatorActor".to_string()),
    )
    .await;

    println!("Actors spawned!\n");

    // Send messages between actors to populate metrics
    println!("Sending messages between actors...");

    // RequestActor sends to WorkerActor
    let (tx, rx) = tokio::sync::oneshot::channel();
    let handler = from_oneshot(tx);
    let msg = sids::actors::messages::Message {
        payload: Some("Process this request".to_string()),
        stop: false,
        responder: Some(handler),
        blocking: None,
    };
    send_message_by_id(&mut actor_system, 1, msg).await?;
    let _ = rx.await;
    actor_system.record_message_sent(0, 1);

    // WorkerActor sends to CoordinatorActor
    let (tx, rx) = tokio::sync::oneshot::channel();
    let handler = from_oneshot(tx);
    let msg = sids::actors::messages::Message {
        payload: Some("Task completed".to_string()),
        stop: false,
        responder: Some(handler),
        blocking: None,
    };
    send_message_by_id(&mut actor_system, 2, msg).await?;
    let _ = rx.await;
    actor_system.record_message_sent(1, 2);

    // CoordinatorActor sends to RequestActor
    let (tx, rx) = tokio::sync::oneshot::channel();
    let handler = from_oneshot(tx);
    let msg = sids::actors::messages::Message {
        payload: Some("All done, results ready".to_string()),
        stop: false,
        responder: Some(handler),
        blocking: None,
    };
    send_message_by_id(&mut actor_system, 0, msg).await?;
    let _ = rx.await;
    actor_system.record_message_sent(2, 0);

    // Send more messages to show activity
    for i in 0..3 {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handler = from_oneshot(tx);
        let msg = sids::actors::messages::Message {
            payload: Some(format!("Message {} from RequestActor", i)),
            stop: false,
            responder: Some(handler),
            blocking: None,
        };
        send_message_by_id(&mut actor_system, 1, msg).await?;
        let _ = rx.await;
        actor_system.record_message_sent(0, 1);
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("Messages sent!\n");

    // Get supervision data
    let supervision = actor_system.get_supervision_data();
    let _summary = actor_system.get_supervision_summary();

    // Display summary
    println!("{}", to_text_summary(&supervision));
    println!();

    // Export to JSON - demonstrate proper error handling
    match to_json(&supervision) {
        Ok(json) => {
            println!("=== JSON Export ===");
            println!("{}\n", json);
        }
        Err(e) => {
            eprintln!("Failed to export JSON: {}", e);
            // Continue execution despite error
        }
    }

    // Export to Graphviz DOT
    let dot = to_dot(&supervision);
    println!("=== Graphviz DOT Export ===");
    println!("{}", dot);
    println!("To visualize this, save to a .dot file and run: dot -Tsvg graph.dot -o graph.svg\n");

    // Export to Mermaid
    let mermaid = to_mermaid(&supervision);
    println!("=== Mermaid Diagram Export ===");
    println!("{}", mermaid);
    println!("Paste the above into https://mermaid.live to visualize\n");

    // Export to Mermaid Sequence Diagram
    let sequence = to_mermaid_sequence(&supervision);
    println!("=== Mermaid Sequence Diagram Export ===");
    println!("{}", sequence);
    println!("Paste the above into https://mermaid.live to visualize the message flow\n");

    // Shutdown with proper error handling
    actor_system.shutdown().await?;
    println!("Actor system shut down gracefully.");

    Ok(())
}
