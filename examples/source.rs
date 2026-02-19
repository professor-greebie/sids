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
    stream_message::NotUsed,
};

#[cfg(feature = "streaming")]
use sids::actors::messages::Message;

fn get_loggings() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

#[cfg(feature = "streaming")]
/// Generate a Source - Sink relationship and send data to the structures.
async fn example_simple_stream() {
    info!("=== Example 1: Simple Source to Sink ===");
    let source = Source::new("Hello, Streaming World!".to_string(), NotUsed);
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
    let mut actor_system = start_actor_system();
    let _materializer = source.to_sink(&mut actor_system, sink).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("Simple stream example completed");
}

#[cfg(feature = "streaming")]
/// Create a stream with a transformative Flow in the middle.
async fn example_stream_with_flow() {
    info!("=== Example 2: Source -> Flow -> Sink ===");
    let source = Source::new(
        "hello from the streaming pipeline".to_string(),
        sids::streaming::stream_message::NotUsed,
    );
    let flow = Flow::new(
        "UppercaseFlow".to_string(),
        |msg: StreamMessage| match msg {
            StreamMessage::Text(text) => StreamMessage::Text(text.to_uppercase()),
            other => other,
        },
    );
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
    let mut actor_system = start_actor_system();
    let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
    
    // Give actors time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    info!("Flow example completed");
}

#[cfg(feature = "streaming")]
async fn example_http_source() {
    info!("=== Example 3: HTTP Source with Error Handling ===");
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
    let data = vec![72, 101, 108, 108, 111]; // "Hello" in bytes
    let source = Source::new(data, sids::streaming::stream_message::NotUsed);
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
    let mut actor_system = start_actor_system();
    let _materializer = source.via_to_sink(&mut actor_system, flow, sink).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[cfg(feature = "streaming")]
async fn example_vector_of_items() {
    info!("=== Example 5: Processing a Vector of Items ===");
    let items = vec!["apple", "banana", "cherry", "date", "elderberry"];
    use sids::streaming::source::SourceActor;
    let stream_messages: Vec<StreamMessage> = items
        .iter()
        .map(|item| StreamMessage::Text(item.to_string()))
        .collect();
    
    let source_actor = SourceActor::new("VectorSource".to_string(), stream_messages);
    use std::sync::{Arc, Mutex};
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    
    let flow = Flow::new(
        "IndexAndUppercaseFlow".to_string(),
        move |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => {
                    let mut count = counter_clone.lock().unwrap();
                    *count += 1;
                    let idx = *count;
                    StreamMessage::Text(format!("Item {}: {}", idx, text.to_uppercase()))
                }
                other => other,
            }
        },
    );
    let sink = Sink::new(
        "VectorSink".to_string(),
        |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => println!("  {}", text),
                StreamMessage::Complete => {
                    println!("  --- All items processed! ---");
                },
                _ => {}
            }
        },
    );
    
    // Start the actor system to help materialize the stream
    let mut actor_system = start_actor_system();
    
    // Spawn sink first
    let sink_id = actor_system.get_actor_count() as u32;
    actor_system.spawn_actor(sink, Some("Sink".to_string())).await;
    let sink_ref = actor_system.get_actor_ref(sink_id);
    
    // Spawn flow
    let mut flow_actor = flow;
    flow_actor.set_downstream(sink_ref.sender.clone());
    let flow_id = actor_system.get_actor_count() as u32;
    actor_system.spawn_actor(flow_actor, Some("Flow".to_string())).await;
    let flow_ref = actor_system.get_actor_ref(flow_id);
    let mut source = source_actor;
    source.set_downstream(flow_ref.sender.clone());
    let source_id = actor_system.get_actor_count() as u32;
    actor_system.spawn_actor(source, Some("Source".to_string())).await;
    let source_ref = actor_system.get_actor_ref(source_id);
    
    source_ref.send(Message {
        payload: Some(StreamMessage::Text("start".to_string())),
        stop: false,
        responder: None,
        blocking: None,
    }).await;
    
    // Give actors time to process all items
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    info!("Vector processing example completed");
}

#[cfg(feature = "streaming")]
async fn example_vector_aggregation() {
    info!("=== Example 6: Vector Processing with Aggregation ===");
    let numbers = vec!["10", "20", "30", "40", "50"];
    
    use sids::streaming::source::SourceActor;
    let stream_messages: Vec<StreamMessage> = numbers
        .iter()
        .map(|num| StreamMessage::Text(num.to_string()))
        .collect();
    
    let source_actor = SourceActor::new("NumberSource".to_string(), stream_messages);
    
    let flow = Flow::new(
        "DoubleFlow".to_string(),
        |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => {
                    if let Ok(num) = text.parse::<i32>() {
                        StreamMessage::Text(format!("{} → {}", num, num * 2))
                    } else {
                        StreamMessage::Error(format!("Invalid number: {}", text))
                    }
                }
                other => other,
            }
        },
    );
    
    // Create a sink that prints the results
    use std::sync::{Arc, Mutex};
    let sum = Arc::new(Mutex::new(0));
    let sum_clone = sum.clone();
    
    let sink = Sink::new(
        "AggregationSink".to_string(),
        move |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => {
                    println!("  {}", text);
                    // Try to extract the doubled value
                    if let Some(arrow_pos) = text.find("→") {
                        if let Some(num_str) = text.get(arrow_pos + 3..) {
                            if let Ok(num) = num_str.trim().parse::<i32>() {
                                *sum_clone.lock().unwrap() += num;
                            }
                        }
                    }
                },
                StreamMessage::Complete => {
                    let total = *sum_clone.lock().unwrap();
                    println!("  --- Total sum of doubled values: {} ---", total);
                },
                StreamMessage::Error(err) => eprintln!("  Error: {}", err),
                _ => {}
            }
        },
    );
    
    // Materialize the stream
    let mut actor_system = start_actor_system();
    
    let sink_id = actor_system.get_actor_count() as u32;
    actor_system.spawn_actor(sink, Some("Sink".to_string())).await;
    let sink_ref = actor_system.get_actor_ref(sink_id);
    
    let mut flow_actor = flow;
    flow_actor.set_downstream(sink_ref.sender.clone());
    let flow_id = actor_system.get_actor_count() as u32;
    actor_system.spawn_actor(flow_actor, Some("Flow".to_string())).await;
    let flow_ref = actor_system.get_actor_ref(flow_id);
    
    let mut source = source_actor;
    source.set_downstream(flow_ref.sender.clone());
    let source_id = actor_system.get_actor_count() as u32;
    actor_system.spawn_actor(source, Some("Source".to_string())).await;
    let source_ref = actor_system.get_actor_ref(source_id);
    
    source_ref.send(sids::actors::messages::Message {
        payload: Some(StreamMessage::Text("start".to_string())),
        stop: false,
        responder: None,
        blocking: None,
    }).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let final_sum = *sum.lock().unwrap();
    println!("Final aggregated sum: {}", final_sum);
    info!("Vector aggregation example completed");
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
        
        example_vector_of_items().await;
        println!();
        
        example_vector_aggregation().await;
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