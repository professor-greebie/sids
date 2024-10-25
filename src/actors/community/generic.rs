use crate::actors::{actor::Actor, messages::{ActorType, Message, NotUsed}};


/// An actor class that does nothing.
/// 
/// This is a dummy actor that does nothing. It is used for testing purposes.
pub(crate) struct Dummy;
impl Actor<NotUsed> for Dummy {
    fn receive(&mut self, _message: Message<ActorType, NotUsed>) {
        // do nothing
    }
}