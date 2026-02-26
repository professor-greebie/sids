/// Example demonstrating supervision tracking for streaming pipelines
///
/// This example shows how the supervision system monitors Source, Flow, and Sink actors
/// in a streaming pipeline, providing visibility into data flow.
#[cfg(feature = "streaming")]
use sids::actors::start_actor_system;

#[cfg(feature = "streaming")]
use sids::streaming::{flow::Flow, sink::Sink, source::Source, stream_message::StreamMessage};

#[cfg(all(feature = "streaming", feature = "visualize"))]
use sids::supervision_export::{to_mermaid, to_mermaid_sequence, to_text_summary};

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("This example requires the 'streaming' feature.");
    println!("Run with: cargo run --example streaming_supervision --features streaming,visualize");
}

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() {
    if !cfg!(feature = "visualize") {
        println!("This example is enhanced with the 'visualize' feature.");
        println!(
            "Run with: cargo run --example streaming_supervision --features streaming,visualize"
        );
        println!("\nRunning without visualization...\n");
    }

    println!("=== Streaming Pipeline with Supervision ===\n");

    // Create a source with sample data strings
    let source = Source::from_items(vec![
        "hello world".to_string(),
        "streaming is awesome".to_string(),
        "rust actors rock".to_string(),
    ]);

    // Create a flow that transforms text to uppercase
    let transform_flow = Flow::new(
        "TransformFlow".to_string(),
        |msg: StreamMessage| match msg {
            StreamMessage::Text(text) => StreamMessage::Text(text.to_uppercase()),
            other => other,
        },
    );

    // Create a sink that prints the results
    let sink = Sink::new("PrintSink".to_string(), |msg: StreamMessage| match msg {
        StreamMessage::Text(text) => {
            println!("→ Processed: {}", text);
        }
        StreamMessage::Complete => println!("✓ Pipeline complete!"),
        _ => {}
    });

    // Create actor system and run the pipeline
    let mut actor_system = start_actor_system();

    println!("Building: Source → TransformFlow → PrintSink\n");

    let _materializer = source
        .via_to_sink(&mut actor_system, transform_flow, sink)
        .await;

    // Allow pipeline to process
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Display supervision data if visualization is enabled
    #[cfg(feature = "visualize")]
    {
        let supervision = actor_system.get_supervision_data();

        println!("\n{}", to_text_summary(&supervision));
        println!();

        // Show flow diagram
        let mermaid = to_mermaid(&supervision);
        println!("=== Mermaid Diagram Export ===");
        println!("{}", mermaid);
        println!("Paste the above into https://mermaid.live to visualize\n");

        // Show sequence diagram
        let sequence = to_mermaid_sequence(&supervision);
        println!("=== Mermaid Sequence Diagram Export ===");
        println!("{}", sequence);
        println!("Paste the above into https://mermaid.live to visualize the message flow\n");
    }

    println!("Streaming pipeline execution completed!");
}
