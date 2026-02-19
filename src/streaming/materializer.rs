use crate::actors::actor_system::ActorSystem;
use crate::actors::actor_ref::ActorRef;
use super::stream_message::StreamMessage;

/// Trait for materializers that orchestrate stream execution
pub trait Materializer {
    fn materialize(&self);
    fn shutdown(&self);
}

/// Implementation of a materializer that runs a stream through the actor system
pub struct StreamMaterializer {
    source_ref: Option<ActorRef<StreamMessage, StreamMessage>>,
    flow_refs: Vec<ActorRef<StreamMessage, StreamMessage>>,
    sink_ref: Option<ActorRef<StreamMessage, StreamMessage>>,
}

impl StreamMaterializer {
    /// Create a new StreamMaterializer
    pub fn new() -> Self {
        StreamMaterializer {
            source_ref: None,
            flow_refs: Vec::new(),
            sink_ref: None,
        }
    }

    /// Set the source actor reference
    pub fn set_source(&mut self, source: ActorRef<StreamMessage, StreamMessage>) {
        self.source_ref = Some(source);
    }

    /// Add a flow actor reference
    pub fn add_flow(&mut self, flow: ActorRef<StreamMessage, StreamMessage>) {
        self.flow_refs.push(flow);
    }

    /// Set the sink actor reference
    pub fn set_sink(&mut self, sink: ActorRef<StreamMessage, StreamMessage>) {
        self.sink_ref = Some(sink);
    }

    /// Get the actor references for debugging
    pub fn get_refs(&self) -> (Option<&ActorRef<StreamMessage, StreamMessage>>, &[ActorRef<StreamMessage, StreamMessage>], Option<&ActorRef<StreamMessage, StreamMessage>>) {
        (self.source_ref.as_ref(), &self.flow_refs, self.sink_ref.as_ref())
    }
}

impl Default for StreamMaterializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Materializer for StreamMaterializer {
    fn materialize(&self) {
        // Materialization happens when the stream is connected
        // This can be extended to perform additional setup
    }

    fn shutdown(&self) {
        // Shutdown logic can be implemented here
        // For now, actors will shut down when their channels are dropped
    }
}