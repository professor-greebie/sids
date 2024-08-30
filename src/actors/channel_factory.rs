use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};

use super::messages::{Message, ResponseMessage};


pub(super) trait ChannelFactory {
    fn create_actor_channel(&self) -> (mpsc::Sender<Message>, mpsc::Receiver<Message>);
    fn create_blocking_actor_channel(&self) -> (Sender<Message>, Receiver<Message>);
    fn create_response_channel(&self) -> (oneshot::Sender<ResponseMessage>, oneshot::Receiver<ResponseMessage>);
    fn create_blocking_response_channel(&self) -> (Sender<ResponseMessage>, Receiver<ResponseMessage>);
}