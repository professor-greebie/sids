
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ActorState {
    Running,
    Started,
    Stopping,
    Stopped
}

pub trait ActorContext<A> {

    fn state(&self) -> ActorState;
    fn parent() -> ActorRef;
    fn children() -> Option<Vec<ActorRef>>;
    fn props() -> Props;
    fn system() -> ActorSystem;
    fn itself() -> ActorRef;
    fn sender() -> ActorRef;

}