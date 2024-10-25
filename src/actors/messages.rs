
use serde::{Deserialize, Serialize};



/// A type that is not used for anything.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NotUsed;
pub struct GuardianType;
pub struct AnyActor;
pub struct BlockingActor;
pub struct Mediator;
pub struct Response;


pub struct Message<T, R> {
    pub for_actor: T,
    pub payload: R,
}

pub enum GuardianMessage {
    

}

pub enum ActorType {
    Mediator(Mediator),
    Officer(AnyActor),
    Blocking(BlockingActor),
    Response(Response),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ResponseMessage {
    SUCCESS, 
    FAILURE,

}

