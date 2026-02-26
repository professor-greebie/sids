use std::fmt;

/// Errors that can occur in the actor system
#[derive(Debug, Clone, PartialEq)]
pub enum ActorError {
    /// Actor with the specified ID was not found
    ActorNotFound { id: u32 },
    /// Failed to send a message to an actor
    SendFailed { reason: String },
    /// Failed to receive a response
    ReceiveFailed { reason: String },
    /// Actor system shutdown timeout
    ShutdownTimeout,
    /// Guardian actor is not responding
    GuardianNotResponding,
    /// Invalid actor state
    InvalidState { reason: String },
    /// Channel operation failed
    ChannelError { reason: String },
}

impl fmt::Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorError::ActorNotFound { id } => {
                write!(f, "Actor with ID {} not found", id)
            }
            ActorError::SendFailed { reason } => {
                write!(f, "Failed to send message: {}", reason)
            }
            ActorError::ReceiveFailed { reason } => {
                write!(f, "Failed to receive response: {}", reason)
            }
            ActorError::ShutdownTimeout => {
                write!(f, "Actor system shutdown timed out")
            }
            ActorError::GuardianNotResponding => {
                write!(f, "Guardian actor is not responding")
            }
            ActorError::InvalidState { reason } => {
                write!(f, "Invalid actor state: {}", reason)
            }
            ActorError::ChannelError { reason } => {
                write!(f, "Channel operation failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for ActorError {}

/// Result type for actor operations
pub type ActorResult<T> = Result<T, ActorError>;
