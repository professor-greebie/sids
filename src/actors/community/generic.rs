use crate::actors::{actor::Actor, messages::Message};


/// An actor class that does nothing.
/// 
/// This is a dummy actor that does nothing. It is used for testing purposes.
pub(crate) struct Dummy;
impl Actor for Dummy {
    fn receive(&mut self, _message: Message) {
        // do nothing
    }
}