extern crate sids;

use sids::actors::community::source::SourceImpl;

#[tokio::main]
async fn main() {
    // Incomplete example for a Source-Flow-Sink actor system
    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let source = SourceImpl::new(1, tx);
    source.run().unwrap();
}

