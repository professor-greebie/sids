


pub struct Source<T> {
    _content: T,
}
/* 
impl<T> Source<T> {
    async fn receive(&mut self, message: InternalMessage) {
        // is it possible to change the message type here?
        SourceTrait::receive(&mut self.content, message);
    }

    fn flatten(&mut self) -> Source<T> {
        let content = std::mem::replace(&mut self.content, Default::default());
        Source::<T> { content }
    }

    fn map<U>(&mut self, f: fn(T) -> U) -> Source<U> {
        Source::<u32> { content: 0 }
    }
} */