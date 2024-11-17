use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};

use super::messages::{Message, ResponseMessage};


pub trait ChannelFactory<R> {
    fn create_actor_channel(&self) -> (mpsc::Sender<Message<R>>, mpsc::Receiver<Message<R>>);
    fn create_blocking_actor_channel(&self) -> (Sender<Message<R>>, Receiver<Message<R>>);
    fn create_response_channel(&self) -> (oneshot::Sender<ResponseMessage>, oneshot::Receiver<ResponseMessage>);
    fn create_blocking_response_channel(&self) -> (Sender<ResponseMessage>, Receiver<ResponseMessage>);
}