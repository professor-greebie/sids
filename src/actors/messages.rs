use super::actor_ref::{ActorRef, BlockingActorRef};



/// Messages used by the guardian actor to communicate with the officers.
///
/// The guardian actor will create the responders to allow for the officers to communicate back to the guardian
/// for status updates and other messages.
#[derive(Debug)]
pub (super) enum Message {
    Terminate,
    OfficerMessage {
        officer_id: u32,
        message: String,
        blocking: bool,
    },
    CreateOfficer {
        officer_type: ActorRef,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    CreateBlockingOfficer {
        officer_type: BlockingActorRef,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    },
    RemoveOfficer {
        officer_id: u32,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    AddCourrier {
        officer_id: u32,
        courrier_type: ActorRef,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
        blocking: bool,
    },
    RemoveCourrier {
        officer_id: u32,
        courrier_id: u32,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
        blocking: bool,
    },
    NotifyCourriers {
        officer_id: u32,
        message: InternalMessage,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
        blocking: bool,
    },

}


/// Internal messages used by the actors to communicate with each other internally.
///
/// All responders created by these items ought to be created by the guardian actor.
#[derive(Debug, Clone, PartialEq)]
pub enum InternalMessage {
    StringMessage {
        message: String,
    },
    Terminate,
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

