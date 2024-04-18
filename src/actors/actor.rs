use std::time::Duration;

pub enum Message {
    SayHello,
    SayGoodbye,
}

pub trait Actor<T> {
    fn receive(&mut self, message: T);
    fn name(&self) -> &str {
        "Unnamed Actor"
    }
}

pub struct TimedActor {
    name: String,
}

impl TimedActor {
    pub fn new(name: &str) -> TimedActor {
        TimedActor {
            name: name.to_string(),
        }
    }
}

pub struct TimedMessage {
    pub message: Message,
    pub timestamp: Duration,
}

impl Actor<TimedMessage> for TimedActor {
    fn receive(&mut self, message: TimedMessage) {
        let TimedMessage { message, timestamp } = message;
        match message {
            Message::SayHello => {
                std::thread::sleep(timestamp);   
                println!(
                    "Hello from actor {}. I waited {:?} seconds.",
                    self.name, timestamp.as_secs());
            }
            Message::SayGoodbye => {
                println!("Goodbye from actor {}", self.name);
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor() {
        let mut actor = TimedActor::new("test_actor");
        assert_eq!(actor.name(), "test_actor");
    }
}
