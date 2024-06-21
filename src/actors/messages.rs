

#[derive(Debug)]
pub enum Message {
    ActorMessage(ActorMessage),
    GetActorMessage(GetActorMessage),
    KafkaProducerMessage(KafkaProducerMessage),
    Response(Response),

}

#[derive(Debug)]
pub enum ActorMessage {
    Terminate,
    GetNextId  {
        responder: tokio::sync::oneshot::Sender<u64>,
    },
}

#[derive(Debug)]

pub enum Response {
    Success, 
    Failure,
}

#[derive(Debug)]

pub enum GetActorMessage {
    Terminate,
    GetURI  {
        uri: String,
        location: String, 
        responder: std::sync::mpsc::Sender<Message>,
    },
}

#[derive(Debug)]

pub enum KafkaProducerMessage {
    Terminate,
    Produce {
        topic: String,
        message: String,
    },
    ProduceWithResponse {
        topic: String,
        message: String,
        responder: tokio::sync::oneshot::Sender<Message>,
    },
}
