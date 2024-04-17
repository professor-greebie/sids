

pub trait Actor<T> {

    fn send<T>(&self, message: T) -> Result<(), String>;
    fn receive<T>(&self, message: T) -> Result<(), String>;
    fn spawn<T>(t: T) -> Result<(), String>;


}