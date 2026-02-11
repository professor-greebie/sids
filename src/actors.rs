
pub mod actor_system;
pub mod actor;
pub mod actor_ref;
pub mod channel_factory;
pub mod messages;

use actor::Actor;
use actor_ref::ActorRef;
use actor_system::ActorSystem;
use channel_factory::ChannelFactory;
use messages::Message;

static SIDS_DEFAULT_BUFFER_SIZE: usize = 100;

pub fn start_actor_system<MType: Send + Clone + 'static, Response: Send + Clone + 'static>() -> ActorSystem<MType, Response> {
    ActorSystem::<MType, Response>::new()
}

pub async fn spawn_actor<MType: Send + Clone + 'static, Response: Send + Clone + 'static, T>(actor_system: &mut ActorSystem<MType, Response>, actor: T, name: Option<String>) where T: Actor<MType, Response> + 'static {  
    actor_system.spawn_actor(actor, name).await;
}

pub async fn spawn_blocking_actor<MType: Send + Clone + 'static, Response: Send + Clone + 'static, T>(actor_system: &mut ActorSystem<MType, Response>, actor: T, name: Option<String>) where T: Actor<MType, Response> + 'static {  
    actor_system.spawn_blocking_actor(actor, name);
}

pub async fn send_message_by_id<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &mut ActorSystem<MType, Response>, actor_id: u32, message: Message<MType, Response>) {
    actor_system.send_message_to_actor(actor_id, message).await;
}

pub async fn ping_actor_system<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) {
    actor_system.ping_system().await;
}

pub fn get_response_channel<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> (tokio::sync::oneshot::Sender<Response>, tokio::sync::oneshot::Receiver<Response>) {
    actor_system.create_response_channel()
}

pub fn get_blocking_response_channel<MType: Send + Clone +'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> (std::sync::mpsc::SyncSender<Response>, std::sync::mpsc::Receiver<Response>) {
    actor_system.create_blocking_response_channel()
}

pub fn get_actor_sender<MType: Send + Clone +'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>, id: u32) -> ActorRef<MType, Response> {
    actor_system.get_actor_ref(id).clone()
}

pub fn get_message_count_reference<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> &'static std::sync::atomic::AtomicUsize {
    actor_system.get_message_count_reference()
}

pub fn get_thread_count_reference<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> &'static std::sync::atomic::AtomicUsize {
    actor_system.get_thread_count_reference()
}

pub fn get_total_messages<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> usize {
    actor_system.get_thread_count()
}

pub fn get_total_threads<MType: Send + Clone + 'static, Response: Send + Clone + 'static>(actor_system: &ActorSystem<MType, Response>) -> usize {
    actor_system.get_message_count()
}

