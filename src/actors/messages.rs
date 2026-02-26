use super::response_handler::BoxedResponseHandler;

#[derive(Debug)]
pub struct Message<R, Response> {
    pub payload: Option<R>,
    pub stop: bool,
    /// Use ResponseHandler trait instead of raw oneshot::Sender to prevent memory leaks
    /// and enable more flexible response handling patterns.
    pub responder: Option<BoxedResponseHandler<Response>>,
    pub blocking: Option<std::sync::mpsc::SyncSender<Response>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseMessage {
    Success,
    Failure { message: Option<String> },
    Status { message: Option<u32> },
    Response { message: String },
    SuccessWithData { data: Option<String> },
    SuccessWithBytes { data: Option<Vec<u8>> },
    FailureWithData { data: Option<String> },
    FailureWithBytes { data: Option<Vec<u8>> },
    InProgress,
    Complete,
    NotFound,
}
