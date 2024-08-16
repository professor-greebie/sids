#[trait_variant::make(SourceTrait<T>: Send)]
trait SourceTraitImpl<T> : ActorTrait {
    async fn receive(&mut self, message: InternalMessage);
    fn flatten(&mut self) -> Source<T>; 
    fn map<U>(&mut self, f: fn(T) -> U) -> Source<U>;

}

pub struct Source<T> {
    content: T,
}

impl SourceTrait<T> for Source<T> {
    async fn receive(&mut self, message: InternalMessage) {
        // is it possible to change the message type here?
        SourceTrait::receive(&mut self.content, message);
    }

    fn flatten(&mut self) -> Source<T> {
        NotImplemented;

    }

    fn map<U>(&mut self, f: fn(T) -> U) -> Source<U> {
        NotImplemented;
    }

    
}