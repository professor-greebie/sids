use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};

use super::messages::Message;


pub trait ChannelFactory<R, Response: Send + Clone + 'static> {
    fn create_actor_channel(&self) -> (mpsc::Sender<Message<R, Response>>, mpsc::Receiver<Message<R, Response>>);
    fn create_blocking_actor_channel(&self) -> (Sender<Message<R, Response>>, Receiver<Message<R, Response>>);
    fn create_response_channel(&self) -> (oneshot::Sender<Response>, oneshot::Receiver<Response>);
    fn create_blocking_response_channel(&self) -> (Sender<Response>, Receiver<Response>);
}