use std::io::Error;
pub enum SourceMessage<T> {
    Send(T),
}

/// A Flow is a reference to a series of actors that are connected in a chain.
pub struct Flow<T, U> {
    _sender: tokio::sync::mpsc::Sender<SourceMessage<U>>,
    _receiver: tokio::sync::mpsc::Receiver<SourceMessage<T>>,
}

impl <T, U> Flow<T, U> {
    pub fn new(_sender: tokio::sync::mpsc::Sender<SourceMessage<U>>, _receiver: tokio::sync::mpsc::Receiver<SourceMessage<T>>) -> Self {
        Flow { _sender, _receiver }
    }
}

/// A Pipeline is a reference to a series of actors that are connected in a chain.
pub struct Pipeline;

/// A Sink is an actor that accepts one input and no output.
/// 
/// It is technically an actor that materializes data sent to it from a source through one of its methods.
pub struct Sink<T> {
    _receiver: tokio::sync::mpsc::Receiver<SourceMessage<T>>,
}

impl<T> Sink<T> {
    async fn receive(&mut self, message: SourceMessage<T>) {
        match message {
            SourceMessage::Send(_content ) => {
                // do nothing
                //
            }
        }
    }

    pub fn new(receiver: tokio::sync::mpsc::Receiver<SourceMessage<T>>) -> Self {
        Sink { _receiver: receiver }
    }
}

async fn run_source<T>(mut actor: Sink<T>) {
    while let Some(message) = actor._receiver.recv().await {
        actor.receive(message).await;
    }
}

/// A source is an actor reference that accepts one input and no output.
pub struct SourceImpl<T>
where
    T: Sized + Send + Clone + 'static,
{
    _content: T,
    _sender: tokio::sync::mpsc::Sender<SourceMessage<T>>,
}

impl<T> SourceImpl<T>
where
    T: Sized + Send + Clone + 'static,
{
    async fn _send(&self) -> Result<(), Error> {
        // need to check that the source is connected to a Sink.
        let cloned_content = self._content.clone();
        let message = SourceMessage::Send(cloned_content);
        self._sender
            .send(message)
            .await
            .expect("Failed to send Source message");
        Ok(())
    }

    pub fn run(&self) -> Result<(), Error> {
        Ok(())
    }

    pub fn new(_content: T, _sender: tokio::sync::mpsc::Sender<SourceMessage<T>>) -> Self {
        SourceImpl { _content, _sender }
    }

    pub fn via<U>(self, _flow: Flow<T, U>) -> Pipeline {

        // connect a flow to the source
        Pipeline
    }

    pub async fn to(self, sink: Sink<T>) -> Pipeline {
        tokio::spawn(
            run_source(sink)
        );
        Pipeline
    }
}
