use std::io::Error;

use super::officer::SelectActor;

#[derive(Debug, Copy, Clone, PartialEq)]

pub enum RefType {
    Tokio,
    Blocking,
}

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

impl Message {
    pub fn to_internal_message(
        self,
        tx: Option<tokio::sync::oneshot::Sender<ResponseMessage>>,
    ) -> Result<InternalMessage, Error> {
        // maybe change this to return a result or an Option<InternalMessage> instead.
        match self {
            Message::Terminate => Ok(InternalMessage::Terminate),
            Message::LogMessage { message } => Ok(InternalMessage::LogMessage { message: message }),
            Message::LogMessageWithResponse { message } => {
                match tx {
                    None => {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot log message without a responder.",
                        ))
                    }
                    _ => {}
                }
                Ok(InternalMessage::LogMessageWithResponse {
                    message: message,
                    responder: tx.unwrap(),
                })
            }
            Message::TerminateById { id } => Ok(InternalMessage::TerminateById { id: id }),
            Message::GetId => Ok(InternalMessage::GetId),
            Message::GetURI {
                uri: _,
                location: _,
            } => {
                // Cannot get URI without a blocking responder.
                Err(Error::new(std::io::ErrorKind::InvalidInput, "Cannot get URI without a blocking responder. Use blocking internal message instead, 
                and include an std::sync::mpsc::Sender."))
                // Need to figure out how to handle the response message
            }
            Message::KafkaProduce {
                topic,
                key,
                message,
            } => {
                Ok(InternalMessage::KafkaProducerMessage(
                    KafkaProducerMessage::Produce {
                        topic: topic,
                        key: key,
                        message: message,
                    },
                ))
                // Need to figure out how to handle the response message
            }
            Message::KafkaConsume { topic: _, group: _ } => {
                // Establish consumption process later
                Ok(InternalMessage::NoMessage)
            }
        }
    }

    pub fn to_blocking_internal_message(
        self,
        tx: Option<std::sync::mpsc::Sender<ResponseMessage>>,
    ) -> Result<InternalMessage, Error> {
        match tx {
            None => return self.to_internal_message(None),
            _ => {}
        }
        match self {
            // maybe change this to return a result or an Option<InternalMessage> instead.
            Message::GetURI { uri, location } => {
                Ok(InternalMessage::CollectorMessage(
                    CollectorMessage::GetURI {
                        uri: uri,
                        location: location,
                        responder: tx.unwrap(),
                    },
                ))
                // Need to figure out how to handle the response message
            }
            _ => Ok(InternalMessage::NoMessage),
        }
    }
}

/// Internal messages used by the actors to communicate with each other internally.
///
/// All responders created by these items ought to be created by the guardian actor.
#[derive(Debug)]
pub enum InternalMessage {
    CollectorMessage(CollectorMessage),
    KafkaProducerMessage(KafkaProducerMessage),
    CleaningActorMessage(CleaningActorMessage),
    GetId,
    NoMessage,
    Terminate,
    TerminateById {
        id: u32,
    },
    LogMessage {
        message: String,
    },
    LogMessageWithResponse {
        message: String,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
}

/// Response messages used by the actors to communicate back to the guardian actor.
///
/// These messages are simplified so that they can be tested easily.
#[derive(Debug, PartialEq)]
pub enum ResponseMessage {
    NoMessage,
    Yes,
    No,
    Terminated,
    Failure,
    Success,
}

/// Messages that are sent to the guardian actor from the actor system.
///
/// The guardian actor will use these messages to create new officers and courriers.
pub(super) enum GuardianMessage {
    Terminate,
    Dispatch {
        officer_id: u32,
        message: Message,
    },
    CreateOfficer {
        officer_type: SelectActor,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    RemoveOfficer {
        officer_id: u32,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    AddCourrier {
        officer_id: u32,
        courrier_type: SelectActor,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    RemoveCourrier {
        officer_id: u32,
        courrier_id: u32,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    #[allow(dead_code)]
    NoMessage,
}

#[derive(Debug)]
pub enum CleaningActorMessage {
    Terminate,
    Clean { location: String },
}

/// Messages to communicate to Collectors.
/// 
/// Collectors are actors that are responsible for getting URIs and writing them to a file.
#[derive(Debug)]
pub enum CollectorMessage {
    Terminate,
    GetURITemplate {
        uri: String,
        location: String,
    },
    GetURI {
        uri: String,
        location: String,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    },
}

/// Messages to communicate to Kafka Producers.
#[derive(Debug)]
pub enum KafkaProducerMessage {
    Terminate,
    Produce {
        topic: String,
        key: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::matches;

    use super::*;

    #[test]
    fn test_message_to_internal_message() {
        let message = Message::LogMessage {
            message: "Test".to_string(),
        };
        let message2 = Message::LogMessageWithResponse {
            message: "Test".to_string(),
        };
        let message3 = Message::Terminate;
        let message4 = Message::TerminateById { id: 1 };
        let message5 = Message::GetId;
        let message6 = Message::GetURI {
            uri: "http://example.com".to_string(),
            location: "test".to_string(),
        };
        let message7 = Message::KafkaProduce {
            topic: "test".to_string(),
            key: "test".to_string(),
            message: "test".to_string(),
        };
        let message8 = Message::KafkaConsume {
            topic: "test".to_string(),
            group: "test".to_string(),
        };
        let (tx, _rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let (tx2, _rx2) = tokio::sync::oneshot::channel::<ResponseMessage>();
        let (tx1, _rx1) = std::sync::mpsc::channel::<ResponseMessage>();
        let (tx3, _rx3) = std::sync::mpsc::channel::<ResponseMessage>();

        let internal_message1 = message.clone().to_internal_message(Some(tx)).unwrap();
        let internal_message1a = message.clone().to_internal_message(None).unwrap();
        let internal_message2 = message2.clone().to_internal_message(Some(tx2)).unwrap();
        let internal_message2a = message2.clone().to_internal_message(None).err().unwrap();

        let internal_message3 = message3.to_internal_message(None).unwrap();
        let internal_message4 = message4.to_internal_message(None).unwrap();
        let internal_message5 = message5.to_internal_message(None).unwrap();
        let internal_message6 = message6
            .clone()
            .to_blocking_internal_message(Some(tx1))
            .unwrap();
        let internal_message6a = message6
            .clone()
            .to_blocking_internal_message(None)
            .err()
            .unwrap();
        let internal_message7 = message7.to_internal_message(None).unwrap();
        let internal_message8 = Message::Terminate
            .to_blocking_internal_message(Some(tx3))
            .unwrap();
        let internal_message9 = message8.to_internal_message(None).unwrap();

        assert!(matches!(internal_message1, InternalMessage::LogMessage { message : _ }));
        assert!(matches!(internal_message1a, InternalMessage::LogMessage { message : _  }));
        assert!(matches!(internal_message2, InternalMessage::LogMessageWithResponse { message : _ , responder: _ }));
        assert!(matches!(internal_message2a.kind(), io::ErrorKind::InvalidInput));
        assert!(matches!(internal_message3, InternalMessage::Terminate));
        assert!(matches!(internal_message4, InternalMessage::TerminateById { id: _  }));
        assert!(matches!(internal_message5, InternalMessage::GetId));
        assert!(matches!(internal_message6, InternalMessage::CollectorMessage(CollectorMessage::GetURI { uri : _ , location: _, responder: _ })));
        assert!(matches!(internal_message6a.kind(), io::ErrorKind::InvalidInput));
        assert!(matches!(internal_message7, InternalMessage::KafkaProducerMessage(KafkaProducerMessage::Produce { topic: _ , key: _ , message: _  })));
        assert!(matches!(internal_message8, InternalMessage::NoMessage));
        assert!(matches!(internal_message9, InternalMessage::NoMessage));
    }
}
