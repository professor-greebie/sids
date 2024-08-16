

/// Messages used by the guardian actor to communicate with the officers.
///
/// The guardian actor will create the responders to allow for the officers to communicate back to the guardian
/// for status updates and other messages.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Terminate,
    TerminateById {
        id: u32,
    },
    GetId,
    GetURI {
        uri: String,
        location: String,
    },
    KafkaProduce {
        topic: String,
        key: String,
        message: String,
    },
    KafkaConsume {
        topic: String,
        group: String,
    },
    LogMessage {
        message: String,
    },
    LogMessageWithResponse {
        message: String,
    },
}


/// Internal messages used by the actors to communicate with each other internally.
///
/// All responders created by these items ought to be created by the guardian actor.
#[derive(Debug, Clone, PartialEq)]
pub struct InternalMessage {
    pub message : Message,
    pub payload : Option<String>,
}

/// Response messages used by the actors to communicate back to the guardian actor.
///
/// These messages are simplified so that they can be tested easil
#[derive(Debug, PartialEq)]
pub enum ResponseMessage {
    NoMessage,
    Yes,
    No,
    Terminated,
    Failure,
    Success,
}

