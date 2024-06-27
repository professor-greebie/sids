// Idea here is that using the logs, we can provide an animation of actors receiving messages and sending messages to each other.

fn get_loggings() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info");
    Builder::from_env(env).init()
}

fn start_sample_actor_system() {
    let mut _actor_system = start_actor_system();
    //_actor_system.spawn_actor(SelectActor::GetActor);
    spawn_kafka_actor(_actor_system);
    let _next_actor_system =_actor_system.next_actor().unwrap()._value.as_ref().unwrap();
    info!("Sending message to get actor reference");
    let _ = _next_actor_system.clone().send(Message::KafkaProducerMessage(KafkaProducerMessage::Produce{
        topic: "junk".to_string(),
        message: "Goodbye".to_string()
    })).await;
}