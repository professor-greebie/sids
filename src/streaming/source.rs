use crate::actors::messages::Message;
use crate::actors::actor::Actor;
use crate::actors::actor_ref::ActorRef;
use crate::actors::actor_system::ActorSystem;
use super::stream_message::{NotUsed, StreamMessage};
use super::materializer::{Materializer, StreamMaterializer};
use super::flow::Flow;
use super::sink::Sink;
use log::info;
use std::time::Duration;

#[cfg(feature = "streaming")]
use reqwest;

#[cfg(feature = "streaming")]
pub struct Source<SourceType, Materializer> {
    mat: Materializer,
    data: Option<SourceType>,
}

impl Default for Source<(), NotUsed> {
    fn default() -> Self {
        Source { mat: NotUsed, data: None }
    }
}

#[derive(Debug)]
pub enum SourceError {
    InvalidUrl(String),
    NetworkError(String),
    InvalidResponse(String),
    Timeout,
    EmptyResponse,
    TooLarge(usize),
}

impl<T, Materializer> Source<T, Materializer> {
    pub fn new(data: T, mat: Materializer) -> Self {
        Source { mat, data: Some(data) }
    }

    pub fn to_materializer(&self) -> &Materializer {
        &self.mat
    }
}

#[cfg(feature = "streaming")]
impl Source<Vec<u8>, NotUsed> {
    /// Creates a source from a URL with safeguards for bad data.
    /// 
    /// `from_url` retrieves data from a URL and creates a source from it.
    /// 
    /// # Arguments
    /// * `url` - The URL to fetch data from
    /// 
    /// # Safeguards
    /// * Validates URL format before making request
    /// * Enforces 30-second timeout
    /// * Checks HTTP status codes
    /// * Limits response size to 10MB
    /// * Validates content is not empty
    /// 
    /// # Returns
    /// * `Ok(Source)` - Successfully fetched data
    /// * `Err(SourceError)` - Failed with detailed error information
    pub async fn from_url(url: &str) -> Result<Self, SourceError> {
        // Validate URL format
        let parsed_url = reqwest::Url::parse(url)
            .map_err(|e| SourceError::InvalidUrl(format!("Invalid URL format: {}", e)))?;

        // Only allow HTTP(S) schemes for safety
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(SourceError::InvalidUrl(
                format!("Only HTTP(S) URLs are supported, got: {}", parsed_url.scheme())
            ));
        }

        // Create client with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SourceError::NetworkError(format!("Failed to build client: {}", e)))?;

        // Make the request
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SourceError::Timeout
                } else {
                    SourceError::NetworkError(format!("Request failed: {}", e))
                }
            })?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(SourceError::InvalidResponse(
                format!("HTTP error: {} - {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        // Check content length if available (10MB limit)
        const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if let Some(content_length) = response.content_length() {
            if content_length > MAX_SIZE as u64 {
                return Err(SourceError::TooLarge(content_length as usize));
            }
        }

        // Get the bytes
        let bytes = response
            .bytes()
            .await
            .map_err(|e| SourceError::NetworkError(format!("Failed to read response body: {}", e)))?;

        // Verify size after download
        if bytes.len() > MAX_SIZE {
            return Err(SourceError::TooLarge(bytes.len()));
        }

        // Check for empty response
        if bytes.is_empty() {
            return Err(SourceError::EmptyResponse);
        }

        Ok(Source {
            mat: NotUsed,
            data: Some(bytes.to_vec()),
        })
    }
}

#[cfg(feature = "streaming")]
impl Source<String, NotUsed> {
    /// Creates a source from a URL and parses response as UTF-8 text.
    /// 
    /// Similar to `from_url` but returns the data as a String with UTF-8 validation.
    pub async fn from_url_text(url: &str) -> Result<Self, SourceError> {
        let bytes_source = Source::<Vec<u8>, NotUsed>::from_url(url).await?;
        
        let text = String::from_utf8(bytes_source.data.unwrap_or_default())
            .map_err(|e| SourceError::InvalidResponse(format!("Invalid UTF-8: {}", e)))?;

        Ok(Source {
            mat: NotUsed,
            data: Some(text),
        })
    }
}

/// SourceActor emits data into the stream
pub struct SourceActor {
    name: String,
    data: Vec<StreamMessage>,
    current_index: usize,
    downstream: Option<tokio::sync::mpsc::Sender<Message<StreamMessage, StreamMessage>>>,
}

impl SourceActor {
    pub fn new(name: String, data: Vec<StreamMessage>) -> Self {
        SourceActor {
            name,
            data,
            current_index: 0,
            downstream: None,
        }
    }

    pub fn set_downstream(&mut self, sender: tokio::sync::mpsc::Sender<Message<StreamMessage, StreamMessage>>) {
        self.downstream = Some(sender);
    }

    /// Emit all data to downstream
    pub async fn emit_all(&mut self) {
        if let Some(downstream) = &self.downstream {
            info!("SourceActor '{}' emitting {} messages", self.name, self.data.len());
            for msg in &self.data {
                let _ = downstream.send(Message {
                    payload: Some(msg.clone()),
                    stop: false,
                    responder: None,
                    blocking: None,
                }).await;
            }
            // Send completion signal
            let _ = downstream.send(Message {
                payload: Some(StreamMessage::Complete),
                stop: false,
                responder: None,
                blocking: None,
            }).await;
            info!("SourceActor '{}' completed emission", self.name);
        }
    }
}

impl Actor<StreamMessage, StreamMessage> for SourceActor {
    async fn receive(&mut self, message: Message<StreamMessage, StreamMessage>) {
        // Source actor can receive control messages
        if let Some(payload) = message.payload {
            match payload {
                StreamMessage::Text(ref cmd) if cmd == "start" => {
                    info!("SourceActor '{}' received start command", self.name);
                    self.emit_all().await;
                }
                _ => {
                    info!("SourceActor '{}' received unexpected message", self.name);
                }
            }
        }
    }
}

#[cfg(feature = "streaming")]
impl Source<Vec<u8>, NotUsed> {
    /// Connect this source to a sink and materialize the stream
    pub async fn to_sink<F>(
        self,
        actor_system: &mut ActorSystem<StreamMessage, StreamMessage>,
        sink: Sink<F>,
    ) -> StreamMaterializer
    where
        F: Fn(StreamMessage) + Send + 'static,
    {
        let mut materializer = StreamMaterializer::new();

        // Convert source data to stream messages
        let data = if let Some(bytes) = self.data {
            vec![StreamMessage::Data(bytes)]
        } else {
            vec![]
        };

        // Create source actor
        let mut source_actor = SourceActor::new("ByteSource".to_string(), data);
        
        // Spawn sink actor
        info!("Spawning sink actor");
        let sink_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(sink, Some("Sink".to_string())).await;
        let sink_ref = actor_system.get_actor_ref(sink_id);
        
        // Set source downstream to sink
        source_actor.set_downstream(sink_ref.sender.clone());
        
        // Spawn source actor
        info!("Spawning source actor");
        let source_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(source_actor, Some("Source".to_string())).await;
        let source_ref = actor_system.get_actor_ref(source_id);
        
        materializer.set_source(source_ref.clone());
        materializer.set_sink(sink_ref);
        
        // Trigger emission
        source_ref.send(Message {
            payload: Some(StreamMessage::Text("start".to_string())),
            stop: false,
            responder: None,
            blocking: None,
        }).await;

        materializer
    }

    /// Connect this source through a flow to a sink
    pub async fn via_to_sink<TransformF, SinkF>(
        self,
        actor_system: &mut ActorSystem<StreamMessage, StreamMessage>,
        flow: Flow<TransformF>,
        sink: Sink<SinkF>,
    ) -> StreamMaterializer
    where
        TransformF: Fn(StreamMessage) -> StreamMessage + Send + 'static,
        SinkF: Fn(StreamMessage) + Send + 'static,
    {
        let mut materializer = StreamMaterializer::new();

        // Convert source data to stream messages
        let data = if let Some(bytes) = self.data {
            vec![StreamMessage::Data(bytes)]
        } else {
            vec![]
        };

        // Create source actor
        let mut source_actor = SourceActor::new("ByteSource".to_string(), data);
        
        // Spawn sink actor first
        info!("Spawning sink actor");
        let sink_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(sink, Some("Sink".to_string())).await;
        let sink_ref = actor_system.get_actor_ref(sink_id);
        
        // Spawn flow actor
        info!("Spawning flow actor");
        let mut flow_actor = flow;
        flow_actor.set_downstream(sink_ref.sender.clone());
        let flow_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(flow_actor, Some("Flow".to_string())).await;
        let flow_ref = actor_system.get_actor_ref(flow_id);
        
        // Set source downstream to flow
        source_actor.set_downstream(flow_ref.sender.clone());
        
        // Spawn source actor
        info!("Spawning source actor");
        let source_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(source_actor, Some("Source".to_string())).await;
        let source_ref = actor_system.get_actor_ref(source_id);
        
        materializer.set_source(source_ref.clone());
        materializer.add_flow(flow_ref);
        materializer.set_sink(sink_ref);
        
        // Trigger emission
        source_ref.send(Message {
            payload: Some(StreamMessage::Text("start".to_string())),
            stop: false,
            responder: None,
            blocking: None,
        }).await;

        materializer
    }
}

#[cfg(feature = "streaming")]
impl Source<String, NotUsed> {
    /// Connect this text source to a sink and materialize the stream
    pub async fn to_sink<F>(
        self,
        actor_system: &mut ActorSystem<StreamMessage, StreamMessage>,
        sink: Sink<F>,
    ) -> StreamMaterializer
    where
        F: Fn(StreamMessage) + Send + 'static,
    {
        let mut materializer = StreamMaterializer::new();

        // Convert source data to stream messages
        let data = if let Some(text) = self.data {
            vec![StreamMessage::Text(text)]
        } else {
            vec![]
        };

        // Create source actor
        let mut source_actor = SourceActor::new("TextSource".to_string(), data);
        
        // Spawn sink actor
        info!("Spawning sink actor");
        let sink_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(sink, Some("Sink".to_string())).await;
        let sink_ref = actor_system.get_actor_ref(sink_id);
        
        // Set source downstream to sink
        source_actor.set_downstream(sink_ref.sender.clone());
        
        // Spawn source actor
        info!("Spawning source actor");
        let source_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(source_actor, Some("Source".to_string())).await;
        let source_ref = actor_system.get_actor_ref(source_id);
        
        materializer.set_source(source_ref.clone());
        materializer.set_sink(sink_ref);
        
        // Trigger emission
        source_ref.send(Message {
            payload: Some(StreamMessage::Text("start".to_string())),
            stop: false,
            responder: None,
            blocking: None,
        }).await;

        materializer
    }

    /// Connect this text source through a flow to a sink
    pub async fn via_to_sink<TransformF, SinkF>(
        self,
        actor_system: &mut ActorSystem<StreamMessage, StreamMessage>,
        flow: Flow<TransformF>,
        sink: Sink<SinkF>,
    ) -> StreamMaterializer
    where
        TransformF: Fn(StreamMessage) -> StreamMessage + Send + 'static,
        SinkF: Fn(StreamMessage) + Send + 'static,
    {
        let mut materializer = StreamMaterializer::new();

        // Convert source data to stream messages
        let data = if let Some(text) = self.data {
            vec![StreamMessage::Text(text)]
        } else {
            vec![]
        };

        // Create source actor
        let mut source_actor = SourceActor::new("TextSource".to_string(), data);
        
        // Spawn sink actor first
        info!("Spawning sink actor");
        let sink_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(sink, Some("Sink".to_string())).await;
        let sink_ref = actor_system.get_actor_ref(sink_id);
        
        // Spawn flow actor
        info!("Spawning flow actor");
        let mut flow_actor = flow;
        flow_actor.set_downstream(sink_ref.sender.clone());
        let flow_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(flow_actor, Some("Flow".to_string())).await;
        let flow_ref = actor_system.get_actor_ref(flow_id);
        
        // Set source downstream to flow
        source_actor.set_downstream(flow_ref.sender.clone());
        
        // Spawn source actor
        info!("Spawning source actor");
        let source_id = actor_system.get_actor_count() as u32;
        actor_system.spawn_actor(source_actor, Some("Source".to_string())).await;
        let source_ref = actor_system.get_actor_ref(source_id);
        
        materializer.set_source(source_ref.clone());
        materializer.add_flow(flow_ref);
        materializer.set_sink(sink_ref);
        
        // Trigger emission
        source_ref.send(Message {
            payload: Some(StreamMessage::Text("start".to_string())),
            stop: false,
            responder: None,
            blocking: None,
        }).await;

        materializer
    }
}