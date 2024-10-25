use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};

use super::messages::{Message, ResponseMessage};


pub(super) trait ChannelFactory {
    fn create_actor_channel<T, R>(&self) -> (mpsc::Sender<Message<T, R>>, mpsc::Receiver<Message<T, R>>);
    fn create_blocking_actor_channel<T, R>(&self) -> (Sender<Message<T, R>>, Receiver<Message<T, R>>);
    fn create_typed_actor_channel<T, R>(&self) -> (mpsc::Sender<Message<T, R>>, mpsc::Receiver<Message<T, R>>);
    fn create_response_channel(&self) -> (oneshot::Sender<ResponseMessage>, oneshot::Receiver<ResponseMessage>);
    fn create_blocking_response_channel(&self) -> (Sender<ResponseMessage>, Receiver<ResponseMessage>);
}