
pub trait ActorMessage {


}

#[derive(PartialEq)]
pub enum GuardianMessage {
    Terminated,
}

impl ActorMessage for GuardianMessage {


}