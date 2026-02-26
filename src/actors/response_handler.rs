use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

/// Trait for handling responses from actors.
/// This allows flexible response handling without requiring 'static lifetimes for raw senders.
/// Using this pattern prevents memory leaks from unhandled oneshot channels.
#[async_trait]
pub trait ResponseHandler<Response: Send + Debug>: Send + Sync + Debug {
    /// Called when a response is received from an actor.
    async fn handle(&self, response: Response);

    /// Called when the response handler is being dropped (timeout, cancellation, etc.)
    /// This allows cleanup of resources.
    fn on_drop(&self) {}
}

/// A simple response handler that sends to a oneshot channel.
/// This is the equivalent of the old oneshot-based approach but wrapped in the handler trait.
pub struct OnehotHandler<Response: Send + Debug> {
    tx: Option<Box<dyn Fn(Response) + Send + Sync>>,
}

impl<Response: Send + Debug + 'static> OnehotHandler<Response> {
    /// Create a new oneshot handler from a closure.
    /// The closure will be called with the response when it arrives.
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(Response) + Send + Sync + 'static,
    {
        OnehotHandler {
            tx: Some(Box::new(handler)),
        }
    }
}

impl<Response: Send + Debug> Debug for OnehotHandler<Response> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnehotHandler").finish()
    }
}

#[async_trait]
impl<Response: Send + Debug + 'static> ResponseHandler<Response> for OnehotHandler<Response> {
    async fn handle(&self, response: Response) {
        if let Some(ref tx) = self.tx {
            tx(response);
        }
    }
}

/// A boxed response handler - allows storing different handler types together.
pub type BoxedResponseHandler<Response> = Arc<dyn ResponseHandler<Response>>;

/// Helper to easily convert a oneshot sender into a boxed response handler.
/// This is the preferred way to upgrade from raw oneshot channels.
pub fn from_oneshot<Response: Send + Debug + 'static>(
    tx: tokio::sync::oneshot::Sender<Response>,
) -> BoxedResponseHandler<Response> {
    Arc::new(OnehotResponseHandler(tokio::sync::Mutex::new(Some(tx))))
}

#[derive(Debug)]
struct OnehotResponseHandler<Response: Send + Debug>(
    tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<Response>>>,
);

#[async_trait]
impl<Response: Send + Debug + 'static> ResponseHandler<Response>
    for OnehotResponseHandler<Response>
{
    async fn handle(&self, response: Response) {
        let mut guard = self.0.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(response);
        }
    }

    fn on_drop(&self) {
        // Oneshot will be automatically cleaned up when the Arc is dropped
    }
}

/// A response handler that collects responses in a channel for batch processing.
/// Useful for high-throughput scenarios where you want to process multiple responses together.
#[derive(Debug)]
pub struct BatchResponseHandler<Response: Send + Debug> {
    tx: tokio::sync::mpsc::Sender<Response>,
}

impl<Response: Send + Debug + 'static> BatchResponseHandler<Response> {
    /// Create a new batch handler with the given buffer size.
    pub fn new(buffer_size: usize) -> (Self, tokio::sync::mpsc::Receiver<Response>) {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer_size);
        (BatchResponseHandler { tx }, rx)
    }
}

#[async_trait]
impl<Response: Send + Debug + 'static> ResponseHandler<Response>
    for BatchResponseHandler<Response>
{
    async fn handle(&self, response: Response) {
        let _ = self.tx.send(response).await;
    }
}

/// A response handler that applies a transformation function to responses.
/// Useful for adapting response types or performing side effects.
pub struct TransformResponseHandler<Response: Send + Debug, T: Send + Debug> {
    handler: Arc<dyn ResponseHandler<T>>,
    transform: Arc<dyn Fn(Response) -> T + Send + Sync>,
}

impl<Response: Send + Debug, T: Send + Debug> Debug for TransformResponseHandler<Response, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransformResponseHandler")
            .field("handler", &self.handler)
            .field("transform", &"<function>")
            .finish()
    }
}

impl<Response: Send + Debug, T: Send + Debug> TransformResponseHandler<Response, T> {
    /// Create a new transform handler.
    pub fn new<F>(handler: Arc<dyn ResponseHandler<T>>, transform: F) -> Self
    where
        F: Fn(Response) -> T + Send + Sync + 'static,
    {
        TransformResponseHandler {
            handler,
            transform: Arc::new(transform),
        }
    }
}

#[async_trait]
impl<Response: Send + Debug + 'static, T: Send + Debug + 'static> ResponseHandler<Response>
    for TransformResponseHandler<Response, T>
{
    async fn handle(&self, response: Response) {
        let transformed = (self.transform)(response);
        self.handler.handle(transformed).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_handler() {
        let (handler, mut rx) = BatchResponseHandler::<String>::new(10);
        let handler = Arc::new(handler);

        handler.handle("Message 1".to_string()).await;
        handler.handle("Message 2".to_string()).await;

        assert_eq!(rx.recv().await, Some("Message 1".to_string()));
        assert_eq!(rx.recv().await, Some("Message 2".to_string()));
    }

    #[tokio::test]
    async fn test_from_oneshot() {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handler = from_oneshot(tx);

        handler.handle(42).await;
        assert_eq!(rx.await, Ok(42));
    }
}
