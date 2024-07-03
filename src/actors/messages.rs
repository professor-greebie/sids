#[derive(Debug)]
pub enum Message {
    ActorMessage(ActorMessage),
    CollectorMessage(CollectorMessage),
    KafkaProducerMessage(KafkaProducerMessage),
    Response(Response),
    NoMessage
}

#[derive(Debug)]
pub enum ActorMessage {
    Terminate,
    GetNextId {
        responder: tokio::sync::oneshot::Sender<u64>,
    },
}

#[derive(Debug)]

pub enum Response {
    Success,
    Failure,
}

#[derive(Debug)]

pub enum CollectorMessage {
    Terminate,
    GetURI {
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