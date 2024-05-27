use crate::actors::actor::ActorMessage;

pub struct ActorRef {
    // possibly include an actor path reference to see where the message is coming from

}

impl ActorRef {
    pub fn tell(&self, message: impl ActorMessage) {
        todo!();
    }
}