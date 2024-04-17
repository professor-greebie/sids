
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ActorState {
    Running,
    Started,
    Stopping,
    Stopped
}

pub trait ActorContext<A> {

    fn state(&self) -> ActorState;

}