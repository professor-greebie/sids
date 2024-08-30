
use super::actor_ref::{ActorRef, BlockingActorRef};



/// Messages used by the guardian actor to communicate with the officers.
///
/// The guardian actor will create the responders to allow for the officers to communicate back to the guardian
/// for status updates and other messages.
#[derive(Debug)]
pub (super) enum GuardianMessage {
    Terminate,
    CountOfficers {
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    CountCourriers {
        officer_id: u32,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    CountActors {
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    OfficerMessage {
        officer_id: u32,
        message: Message,
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
        message: Message,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
        blocking: bool,
    },

}


/// Internal messages used by the actors to communicate with each other internally.
///
/// All responders created by these items ought to be created by the guardian actor.
#[derive(Debug)]
pub enum Message {
    StringMessage {
        message: String,
    },
    StringResponse {
        message: String,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    StringResponseBlocking {
        message: String,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    },
    IntegerMessage {
        number: i32,
    },
    IntegerResponse {
        number: i32,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    IntegerResponseBlocking {
        number: i32,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    },
    DoubleMessage {
        number: f64,
    },
    DoubleResponse {
        number: f64,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    DoubleResponseBlocking {
        number: f64,
        responder: std::sync::mpsc::Sender<ResponseMessage>,
    },
    RequestStatus {
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    GetUrl {
        url: String,
        output: String,
        responder: tokio::sync::oneshot::Sender<ResponseMessage>,
    },
    UpdateName {
        name: String,
    },
    Terminate,
}

/// Response messages used by the actors to communicate back to the guardian actor.
///
/// These messages are simplified so that they can be tested easil
#[derive(Debug, PartialEq)]
pub enum ResponseMessage {
    NoMessage,
    Ok,
    Problem(String),
    Yes,
    No,
    Terminated,
    Failure,
    Success,
    Response(String),
    ResponseCount(String, u32),

}

