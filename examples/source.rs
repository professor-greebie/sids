extern crate sids;

use env_logger::{Builder, Env};
use log::info;

#[cfg(feature = "streaming")]
use sids::actors::start_actor_system;

#[cfg(feature = "streaming")]
use sids::streaming::{
    source::Source,
    flow::Flow,
    sink::Sink,
    stream_message::StreamMessage,
};

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

#[cfg(feature = "streaming")]
async fn example_simple_stream() {
    info!("=== Example 1: Simple Source to Sink ===");
    
    // Create a simple text source
    let source = Source::new("Hello, Streaming World!".to_string(), sids::streaming::stream_message::NotUsed);
    
    // Create a sink that prints to console
    let sink = Sink::new(
        "PrintSink".to_string(),
        |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => println!("Received: {}", text),
                StreamMessage::Complete => println!("Stream completed!"),
                _ => {}
            }
        },
    );
    
    // Create actor system and materialize the stream
    let mut actor_system = start_actor_system();
    let _materializer = source.to_sink(&mut actor_system, sink).await;
    
    // Give actors time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("Simple stream example completed");
}

#[cfg(feature = "streaming")]
async fn example_stream_with_flow() {
    info!("=== Example 2: Source -> Flow -> Sink ===");
    
    // Create a text source
    let source = Source::new(
        "hello from the streaming pipeline".to_string(),
        sids::streaming::stream_message::NotUsed,
    );
    
    // Create a flow that converts text to uppercase
    let flow = Flow::new(
        "UppercaseFlow".to_string(),
        |msg: StreamMessage| match msg {
            StreamMessage::Text(text) => StreamMessage::Text(text.to_uppercase()),
            other => other,
        },
    );
    
    // Create a sink that prints the transformed data
    let sink = Sink::new(
        "OutputSink".to_string(),
        |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => println!("Transformed: {}", text),
                StreamMessage::Complete => println!("Transformation complete!"),
                _ => {}
            }
        },
    );
    
    // Create actor system and materialize the stream with flow
    let mut actor_system = start_actor_system();
    let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
    
    // Give actors time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("Flow example completed");
}

#[cfg(feature = "streaming")]
async fn example_http_source() {
    info!("=== Example 3: HTTP Source with Error Handling ===");
    
    // Try to fetch data from a URL (using a simple API endpoint)
    match Source::from_url_text("https://httpbin.org/get").await {
        Ok(source) => {
            info!("Successfully fetched data from URL");
            
            // Create a flow that extracts just the first 100 characters
            let flow = Flow::new(
                "TruncateFlow".to_string(),
                |msg: StreamMessage| match msg {
                    StreamMessage::Text(text) => {
                        let truncated = if text.len() > 100 {
                            format!("{}...", &text[..100])
                        } else {
                            text
                        };
                        StreamMessage::Text(truncated)
                    }
                    other => other,
                },
            );
            
            // Create a sink that prints the data
            let sink = Sink::new(
                "HttpSink".to_string(),
                |msg: StreamMessage| {
                    match msg {
                        StreamMessage::Text(text) => println!("HTTP Response Preview: {}", text),
                        StreamMessage::Complete => println!("HTTP stream completed!"),
                        StreamMessage::Error(err) => eprintln!("Error: {}", err),
                        _ => {}
                    }
                },
            );
            
            // Materialize the stream
            let mut actor_system = start_actor_system();
            let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
            
            // Give actors time to process
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        Err(e) => {
            eprintln!("Failed to fetch from URL: {:?}", e);
            info!("This is expected if there's no network connection");
        }
    }
}

#[cfg(feature = "streaming")]
async fn example_byte_source() {
    info!("=== Example 4: Binary Data Processing ===");
    
    // Create a byte source
    let data = vec![72, 101, 108, 108, 111]; // "Hello" in bytes
    let source = Source::new(data, sids::streaming::stream_message::NotUsed);
    
    // Create a flow that converts bytes to text
    let flow = Flow::new(
        "ByteToTextFlow".to_string(),
        |msg: StreamMessage| match msg {
            StreamMessage::Data(bytes) => {
                match String::from_utf8(bytes) {
                    Ok(text) => StreamMessage::Text(format!("Decoded: {}", text)),
                    Err(_) => StreamMessage::Error("Invalid UTF-8".to_string()),
                }
            }
            other => other,
        },
    );
    
    // Create a sink
    let sink = Sink::new(
        "ByteSink".to_string(),
        |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => println!("{}", text),
                StreamMessage::Error(err) => eprintln!("Error: {}", err),
                StreamMessage::Complete => println!("Byte processing complete!"),
                _ => {}
            }
        },
    );
    
    // Materialize
    let mut actor_system = start_actor_system();
    let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
    
    // Give actors time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

async fn start_sample_actor_system() {
    #[cfg(feature = "streaming")]
    {
        example_simple_stream().await;
        println!();
        
        example_stream_with_flow().await;
        println!();
        
        example_byte_source().await;
        println!();
        
        // Optional: try HTTP example (may fail without network)
        example_http_source().await;
    }
    
    #[cfg(not(feature = "streaming"))]
    {
        println!("Streaming feature is not enabled. Build with --features streaming");
    }
}

#[tokio::main]
async fn main() {
    get_loggings();
    info!("Starting streaming examples...");
    start_sample_actor_system().await;
    info!("All examples completed!");
}