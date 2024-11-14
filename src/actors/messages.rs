
use serde::{Deserialize, Serialize};



/// A type that is not used for anything.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NotUsed;

#[derive(Debug)]
pub struct Message<R> {
    pub payload: Option<R>,
    pub stop : bool,
    pub responder: Option<tokio::sync::oneshot::Sender<ResponseMessage>>,
    pub blocking: Option<std::sync::mpsc::Sender<ResponseMessage>>,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ResponseMessage {
    SUCCESS, 
    FAILURE,

}

