use super::materializer::Materializer;

/// Structure messages with no reply expected.
/// Sources will not use a reply message channel, for instance.
#[derive(Debug, Clone, Copy)]
pub struct NotUsed;

impl Materializer for NotUsed {
    fn materialize(&self) {
        // do nothing
    }
    fn shutdown(&self) {
        // do nothing
    }
}

/// Messages that flow through the stream between actors
#[derive(Debug, Clone)]
pub enum StreamMessage {
    Data(Vec<u8>),
    Text(String),
    Complete,
    Error(String),
}

impl StreamMessage {
    pub fn is_terminal(&self) -> bool {
        matches!(self, StreamMessage::Complete | StreamMessage::Error(_))
    }

    pub fn as_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            StreamMessage::Data(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            StreamMessage::Text(text) => Some(text),
            _ => None,
        }
    }
}
