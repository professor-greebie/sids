#[derive(Debug)]
pub struct Message<R, Response> {
    pub payload: Option<R>,
    pub stop : bool,
    pub responder: Option<tokio::sync::oneshot::Sender<Response>>,
    pub blocking: Option<std::sync::mpsc::Sender<Response>>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum ResponseMessage {
    Success, 
    Failure {
        message: Option<String>,
    },
    Status {
        message: Option<u32>,
    },
    Response {
        message: String,
    },
    SuccessWithData {
        data: Option<String>,
    },
    SuccessWithBytes {
        data: Option<Vec<u8>>,
    },
    FailureWithData {
        data: Option<String>,
    },
    FailureWithBytes {
        data: Option<Vec<u8>>,
    },
    InProgress,
    Complete,
    NotFound,

}

