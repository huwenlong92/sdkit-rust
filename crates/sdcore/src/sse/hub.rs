use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::SseMessage;

pub type SseReceiver = UnboundedReceiver<SseMessage>;

#[derive(Clone, Default)]
pub struct SseHub {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

struct Subscriber {
    topic: String,
    sender: UnboundedSender<SseMessage>,
}

impl SseHub {
    pub fn subscribe(&self, topic: impl Into<String>) -> SseReceiver {
        let (sender, receiver) = mpsc::unbounded_channel();
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.push(Subscriber {
                topic: topic.into(),
                sender,
            });
        }
        receiver
    }

    pub fn publish(&self, topic: &str, message: SseMessage) {
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.retain(|subscriber| {
                if subscriber.topic != topic {
                    return true;
                }
                subscriber.sender.send(message.clone()).is_ok()
            });
        }
    }
}
