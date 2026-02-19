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
    use std::sync::{Arc, Mutex};

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
    #[allow(clippy::type_complexity)]
    pub fn create_collector() -> (impl Fn(StreamMessage), Arc<Mutex<Vec<Vec<u8>>>>) {
        let storage = Arc::new(Mutex::new(Vec::new()));
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

    /// Create a consumer that applies a function to each line of text
    /// 
    /// For StreamMessage::Text, splits by lines and applies the function to each line.
    /// For StreamMessage::Data, converts to UTF-8 if possible and processes lines.
    pub fn for_each<F>(f: F) -> impl Fn(StreamMessage)
    where
        F: Fn(&str) + Send + 'static,
    {
        move |msg: StreamMessage| {
            match msg {
                StreamMessage::Text(text) => {
                    for line in text.lines() {
                        f(line);
                    }
                },
                StreamMessage::Data(bytes) => {
                    if let Ok(text) = String::from_utf8(bytes) {
                        for line in text.lines() {
                            f(line);
                        }
                    }
                },
                StreamMessage::Complete => {},
                StreamMessage::Error(_) => {},
            }
        }
    }

    /// Create a consumer that applies a function to each message
    /// 
    /// Unlike for_each which processes line-by-line, this processes entire messages
    pub fn for_each_message<F>(f: F) -> impl Fn(StreamMessage)
    where
        F: Fn(StreamMessage) + Send + 'static,
    {
        move |msg: StreamMessage| {
            f(msg);
        }
    }
}