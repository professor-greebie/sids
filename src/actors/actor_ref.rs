use crate::actors::actor::ActorMessage;

#[derive(Eq, PartialEq, Hash)]
pub struct ActorRef {
    pub name: String,
    // possibly include an actor path reference to see where the message is coming from

}

impl ActorRef {
    pub fn tell(&self, message: impl ActorMessage) {
        todo!();
    }
    pub fn stop(&self) {
        todo!();
    }
}