use super::{actor_ref::ActorRef, officer::SelectActor};

#[derive(Debug, Copy, Clone, PartialEq)]

pub enum RefType {
    Tokio,
    Blocking
}


/// Messages used by the guardian actor to communicate with the officers.
/// 
/// The guardian actor will create the responders to allow for the officers to communicate back to the guardian
/// for status updates and other messages.
#[derive(Debug)]
pub enum Message {
    Terminate, 
    GetId,
    GetURITemplate {
        uri: String,
        location: String
    },

}


/// Internal messages used by the actors to communicate with each other internally.
/// 
/// All responders created by these items ought to be created by the guardian actor.
#[derive(Debug)]
pub enum InternalMessage {
    ActorMessage(ActorMessage),
    CollectorMessage(CollectorMessage),
    KafkaProducerMessage(KafkaProducerMessage),
    CleaningActorMessage(CleaningActorMessage),
    NoMessage,
    Terminate
}


/// Response messages used by the actors to communicate back to the guardian actor.
/// 
/// These messages are simplified so that they can be tested easily.
#[derive(Debug, PartialEq)]
pub enum ResponseMessage {
    Response(Response),
    NoMessage,
    Yes,
    No,
    Terminated,
    Failure,
    Success

}


/// Messages that are sent to the guardian actor from the actor system.
/// 
/// The guardian actor will use these messages to create new officers and courriers.
pub (super) enum GuardianMessage {
    Terminate, 
    Dispatch {
        officer_id: u32,
        ref_type: RefType,
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
}

#[derive(Debug)]
pub enum CleaningActorMessage {
    Terminate,
    Clean {
        location: String,
    },
}

#[derive(Debug)]
pub enum ActorMessage {
    Terminate,
    GetNextId,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum Response {
    Success,
    Failure,
}

#[derive(Debug)]

pub enum CollectorMessage {
    Terminate,
    GetURITemplate {
        uri: String,
        location: String
    },
    GetURI {
        uri: String,
        location: String,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    },
}

#[derive(Debug)]

pub enum KafkaProducerMessage {
    Terminate,
    Produce {
        topic: String,
        message: String,
    },
}