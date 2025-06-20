#[derive(Debug)]
pub struct Message<R> {
    pub payload: Option<R>,
    pub stop : bool,
    pub responder: Option<tokio::sync::oneshot::Sender<ResponseMessage>>,
    pub blocking: Option<std::sync::mpsc::Sender<ResponseMessage>>,
}


#[derive(Debug, PartialEq)]
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

