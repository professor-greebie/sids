
pub trait ActorMessage {
}

pub enum GuardianMessage {
    Terminated,
    GetNextId {
        responder: tokio::sync::oneshot::Sender<u64>,
    }
}

impl ActorMessage for GuardianMessage {


}