use std::sync::mpsc::{Receiver, Sender, SyncSender};
use tokio::sync::{mpsc, oneshot};

use super::messages::Message;

type AsyncActorChannel<R, Response> = (
    mpsc::Sender<Message<R, Response>>,
    mpsc::Receiver<Message<R, Response>>,
);
type BlockingActorChannel<R, Response> =
    (Sender<Message<R, Response>>, Receiver<Message<R, Response>>);

pub trait ChannelFactory<R, Response: Send + Clone + 'static> {
    fn create_actor_channel(&self) -> AsyncActorChannel<R, Response>;
    fn create_blocking_actor_channel(&self) -> BlockingActorChannel<R, Response>;
    fn create_response_channel(&self) -> (oneshot::Sender<Response>, oneshot::Receiver<Response>);
    fn create_blocking_response_channel(&self) -> (SyncSender<Response>, Receiver<Response>);
}
