use std::thread::available_parallelism;

pub mod executor;


/// A Runnable is a wrapper for a function that can be executed by an Executor.
trait Runnable {
    fn run();
}

trait Executor {
    fn execute(runnable: Runnable);
}

trait ActorSystemExecutor: Executor {
    fn execute(runnable: Runnable);
    fn report_failure(cause: String);

}

enum ActorSystemExecutorFactory {
    default,
    thread_pool,
    multi_processor_single_consumer,
    

}

impl ActorSystemExecutorFactory {
    fn create() -> Box<dyn ActorSystemExecutor> {
        match self {
            ActorSystemExecutorFactory::default => Box::new(DefaultActorSystemExecutor::new()),
            ActorSystemExecutorFactory::thread_pool => Box::new(ThreadPoolActorSystemExecutor::new()),
            ActorSystemExecutorFactory::multi_processor_single_consumer => Box::new(MultiProcessorSingleConsumerActorSystemExecutor::new()),
        }
    }
}

struct DefaultActorSystemExecutor {
    name: String,
    parallelism: usize,
}