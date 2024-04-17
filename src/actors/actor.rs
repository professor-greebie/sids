

pub trait Actor {

    fn receive(&self, message: Message) -> Result<(), String>;
    fn send(&self, message: Message) -> Result<(), String>;
    fn spawn(&self) -> Result<(), String>;

}