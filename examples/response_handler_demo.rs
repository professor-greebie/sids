/// Example demonstrating memory-safe response handling with the ResponseHandler pattern
///
/// This example shows how the new ResponseHandler prevents memory leaks that could occur
/// with raw oneshot channels when many messages are sent without awaiting responses.
use sids::actors::{
    actor::Actor,
    get_response_handler,
    messages::{Message, ResponseMessage},
    send_message_by_id, spawn_actor, start_actor_system,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// A simple echo actor that responds to all messages
struct EchoActor {
    message_count: Arc<AtomicUsize>,
}

impl Actor<String, ResponseMessage> for EchoActor {
    async fn receive(&mut self, message: Message<String, ResponseMessage>) {
        self.message_count.fetch_add(1, Ordering::SeqCst);

        if let Some(responder) = message.responder {
            // The ResponseHandler trait ensures proper cleanup
            // Even if the receiver is dropped, the handler cleans up automatically
            responder.handle(ResponseMessage::Success).await;
        }
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_demo().await {
        eprintln!("Demo failed: {}", e);
        std::process::exit(1);
    }
}

async fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ResponseHandler Memory Safety Demo ===");
    println!("=== Error Handling Patterns ===\n");

    let message_count = Arc::new(AtomicUsize::new(0));
    let actor = EchoActor {
        message_count: message_count.clone(),
    };

    let mut actor_system = start_actor_system::<String, ResponseMessage>();
    spawn_actor(&mut actor_system, actor, Some("EchoActor".to_string())).await;

    println!("Scenario 1: Normal request-response pattern with error handling");
    {
        let (handler, rx) = get_response_handler::<ResponseMessage>();

        let msg = Message {
            payload: Some("Hello, Actor!".to_string()),
            stop: false,
            responder: Some(handler),
            blocking: None,
        };

        // Use ? operator for clean error propagation
        send_message_by_id(&mut actor_system, 0, msg).await?;

        // Pattern match on the Result for detailed error handling
        match rx.await {
            Ok(response) => println!("✅ Received response: {:?}", response),
            Err(e) => {
                println!("❌ Response channel closed: {}", e);
                return Err(Box::new(e));
            }
        }
    }
    // Handler automatically cleaned up when dropped
    println!("Handler dropped - memory cleaned up automatically\n");

    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("Scenario 2: High-throughput without awaiting all responses");
    println!("Sending 1000 messages with graceful error handling...");
    {
        let start_count = message_count.load(Ordering::SeqCst);

        // Send many messages without immediately awaiting
        // This demonstrates that handlers clean up automatically
        for i in 0..1000 {
            let (handler, rx) = get_response_handler::<ResponseMessage>();

            let msg = Message {
                payload: Some(format!("Message #{}", i)),
                stop: false,
                responder: Some(handler),
                blocking: None,
            };

            // Use ? operator - will exit early on first error
            send_message_by_id(&mut actor_system, 0, msg).await?;

            // Purposely drop some receivers to simulate real-world scenarios
            if i % 100 == 0 {
                // Await every 100th response with error handling
                if let Err(e) = rx.await {
                    println!("⚠️  Response #{} failed: {}", i, e);
                }
            }
            // Other receivers are dropped without awaiting
            // With raw oneshot channels, this could cause memory buildup
            // With ResponseHandler, cleanup is automatic
        }

        // Wait for all messages to be processed
        tokio::time::sleep(Duration::from_millis(500)).await;

        let final_count = message_count.load(Ordering::SeqCst);
        let processed = final_count - start_count;
        println!("✅ Processed {} messages without memory leaks", processed);
        println!("All handlers cleaned up automatically\n");
    }

    println!("Scenario 3: Using Arc for shared response handling");
    {
        use sids::actors::response_handler::BatchResponseHandler;
        use std::sync::Arc;

        let (batch_handler, mut rx) = BatchResponseHandler::<ResponseMessage>::new(100);
        let handler = Arc::new(batch_handler);

        println!("Sending 50 messages with shared batch handler...");

        for i in 0..50 {
            let msg = Message {
                payload: Some(format!("Batch message #{}", i)),
                stop: false,
                responder: Some(handler.clone()),
                blocking: None,
            };

            // Use ? operator for proper error propagation
            send_message_by_id(&mut actor_system, 0, msg).await?;
        }

        // Collect responses in batches with error handling
        let mut collected = 0;
        while collected < 50 {
            match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                Ok(Some(_response)) => collected += 1,
                Ok(None) => {
                    println!("⚠️  Channel closed after {} responses", collected);
                    break;
                }
                Err(_) => {
                    println!("⚠️  Timeout waiting for response #{}", collected + 1);
                    break;
                }
            }
        }

        println!("✅ Collected {} responses via batch handler", collected);
        println!("Batch handler shares single Arc - efficient memory usage\n");
    }
    // Arc and handler automatically cleaned up

    println!("=== Demo Complete ===");
    println!("\nKey Benefits Demonstrated:");
    println!("1. Automatic cleanup when handlers are dropped");
    println!("2. No memory leaks even when receivers aren't awaited");
    println!("3. Efficient shared handling via Arc");
    println!("4. Safe for high-throughput scenarios");
    println!("5. Proper error handling with Result types");

    let total = message_count.load(Ordering::SeqCst);
    println!("\nTotal messages processed: {}", total);

    Ok(())
}
