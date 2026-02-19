use crate::actors::actor::Actor;
use crate::actors::messages::Message;
use super::stream_message::StreamMessage;
use log::info;
use std::sync::{Arc, Mutex};

/// A Sink is the terminal point of a stream that consumes data.
/// Sinks are implemented as actors that receive and process final data.
pub struct Sink<F>
where
    F: Fn(StreamMessage) + Send + 'static,
{
    name: String,
    consumer: F,
    messages_received: Arc<Mutex<usize>>,
}

impl<F> Sink<F>
where
    F: Fn(StreamMessage) + Send + 'static,
{
    /// Create a new Sink with a consumption function
    pub fn new(name: String, consumer: F) -> Self {
        Sink {
            name,
            consumer,
            messages_received: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the count of messages received by this sink
    pub fn messages_received(&self) -> usize {
        *self.messages_received.lock().unwrap()
    }
}

impl<F> Actor<StreamMessage, StreamMessage> for Sink<F>
where
    F: Fn(StreamMessage) + Send + 'static,
{
    async fn receive(&mut self, message: Message<StreamMessage, StreamMessage>) {
        if let Some(payload) = message.payload {
            *self.messages_received.lock().unwrap() += 1;
            info!("Sink '{}' received message #{}", self.name, self.messages_received());

            // Consume the data
            (self.consumer)(payload.clone());

            // If terminal message, log completion
            if payload.is_terminal() {
                info!("Sink '{}' received terminal message, processing complete", self.name);
            }
        }
    }
}

/// Predefined sink consumers
pub mod consumers {
    use super::StreamMessage;
    use log::info;

    /// Print data to console
    pub fn print_console(msg: StreamMessage) {
        match msg {
            StreamMessage::Data(bytes) => {
                info!("Sink received {} bytes of data", bytes.len());
            }
            StreamMessage::Text(text) => {
                println!("Sink output: {}", text);
            }
            StreamMessage::Complete => {
                info!("Stream completed successfully");
            }
            StreamMessage::Error(err) => {
                eprintln!("Stream error: {}", err);
            }
        }
    }

    /// Collect data into a vector (requires external storage)
    pub fn create_collector() -> (impl Fn(StreamMessage), std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>) {
        let storage = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let storage_clone = storage.clone();
        
        let collector = move |msg: StreamMessage| {
            if let StreamMessage::Data(bytes) = msg {
                storage_clone.lock().unwrap().push(bytes);
            }
        };
        
        (collector, storage)
    }

    /// Ignore all data (useful for testing)
    pub fn ignore(_msg: StreamMessage) {
        // Do nothing
    }
}