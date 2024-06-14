

pub trait Message {

}

pub enum ActorMessage {
    Terminate,
    GetNextId  {
        responder: tokio::sync::oneshot::Sender<u64>,
    },
}

pub enum GetActorMessage {
    Terminate,
    GetURI  {
        uri: String,
        location: String,
    },
}

impl Message for GetActorMessage {
    
}