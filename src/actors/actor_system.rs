
use tokio::sync::mpsc;
/// An actor system is a collection of actors that can communicate with each other.
struct ActorSystem {
    actors: HashMap<String, thread::JoinHandle<()>>,
}

impl ActorSystem {
    fn new() -> ActorSystem {
        ActorSystem {
            actors: HashMap::new(),
        }
    }
    fn spawn<A, M>(&mut self, actor: A)
    where
        A: Actor<M> + 'static,
        M: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let actor_ref = Arc::new(Mutex::new(Some(sender)));
        let handle = thread::spawn(move || {
            actor.start();
            loop {
                let message = receiver.recv().unwrap();
                actor.receive(message);
            }
        });
        self.actors.insert(actor.name(), handle);
    }
}