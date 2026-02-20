use crate::actors::actor::Actor;
use crate::actors::messages::Message;
use super::stream_message::StreamMessage;
use log::info;

/// A Flow transforms data as it passes through the stream.
/// Flows are implemented as actors that receive data, transform it, and send it downstream.
pub struct Flow<F>
where
    F: Fn(StreamMessage) -> StreamMessage + Send + 'static,
{
    name: String,
    transform: F,
    downstream: Option<tokio::sync::mpsc::Sender<Message<StreamMessage, StreamMessage>>>,
}

impl<F> Flow<F>
where
    F: Fn(StreamMessage) -> StreamMessage + Send + 'static,
{
    /// Create a new Flow with a transformation function
    pub fn new(name: String, transform: F) -> Self {
        Flow {
            name,
            transform,
            downstream: None,
        }
    }

    /// Set the downstream actor to send transformed data to
    pub fn set_downstream(&mut self, sender: tokio::sync::mpsc::Sender<Message<StreamMessage, StreamMessage>>) {
        self.downstream = Some(sender);
    }

    /// Chain another transformation to this flow
    /// 
    /// Creates a new Flow that applies both transformations in sequence
    pub fn map<G>(self, g: G) -> Flow<impl Fn(StreamMessage) -> StreamMessage + Send + 'static>
    where
        G: Fn(StreamMessage) -> StreamMessage + Send + 'static,
    {
        let name = format!("{}_mapped", self.name);
        let f = self.transform;
        Flow::new(name, move |msg| g(f(msg)))
    }

    /// Add a filter to this flow
    /// 
    /// Messages that don't pass the predicate are converted to Error messages
    pub fn filter<P>(self, predicate: P) -> Flow<impl Fn(StreamMessage) -> StreamMessage + Send + 'static>
    where
        P: Fn(&StreamMessage) -> bool + Send + 'static,
    {
        let name = format!("{}_filtered", self.name);
        let f = self.transform;
        Flow::new(name, move |msg| {
            let transformed = f(msg);
            if predicate(&transformed) {
                transformed
            } else {
                StreamMessage::Error("Filtered out".to_string())
            }
        })
    }
}

impl<F> Actor<StreamMessage, StreamMessage> for Flow<F>
where
    F: Fn(StreamMessage) -> StreamMessage + Send + 'static,
{
    async fn receive(&mut self, message: Message<StreamMessage, StreamMessage>) {
        if let Some(payload) = message.payload {
            info!("Flow '{}' received message", self.name);

            // Check if this is a terminal message
            if payload.is_terminal() {
                info!("Flow '{}' received terminal message, forwarding downstream", self.name);
                if let Some(downstream) = &self.downstream {
                    let _ = downstream.send(Message {
                        payload: Some(payload),
                        stop: false,
                        responder: None,
                        blocking: None,
                    }).await;
                }
                return;
            }

            // Apply transformation
            let transformed = (self.transform)(payload);
            info!("Flow '{}' transformed data", self.name);

            // Send transformed data downstream
            if let Some(downstream) = &self.downstream {
                let _ = downstream.send(Message {
                    payload: Some(transformed),
                    stop: false,
                    responder: None,
                    blocking: None,
                }).await;
            }
        }
    }
}

/// Predefined transformation functions
pub mod transforms {
    use super::StreamMessage;

    /// Convert bytes to uppercase text (if valid UTF-8)
    pub fn to_uppercase(msg: StreamMessage) -> StreamMessage {
        match msg {
            StreamMessage::Data(bytes) => {
                if let Ok(text) = String::from_utf8(bytes) {
                    StreamMessage::Text(text.to_uppercase())
                } else {
                    StreamMessage::Error("Invalid UTF-8 data".to_string())
                }
            }
            StreamMessage::Text(text) => StreamMessage::Text(text.to_uppercase()),
            other => other,
        }
    }

    /// Filter out empty messages
    pub fn filter_empty(msg: StreamMessage) -> StreamMessage {
        match &msg {
            StreamMessage::Data(bytes) if bytes.is_empty() => {
                StreamMessage::Error("Empty data".to_string())
            }
            StreamMessage::Text(text) if text.is_empty() => {
                StreamMessage::Error("Empty text".to_string())
            }
            _ => msg,
        }
    }

    /// Pass through without transformation
    pub fn identity(msg: StreamMessage) -> StreamMessage {
        msg
    }
}