use super::SseMessage;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Bounded topics with explicit gap notifications for slow subscribers.
#[derive(Clone)]
pub struct BoundedSseHub {
    topics: Arc<Mutex<std::collections::HashMap<String, broadcast::Sender<SseMessage>>>>,
    capacity: usize,
}

pub struct BoundedSseReceiver {
    receiver: broadcast::Receiver<SseMessage>,
}

impl BoundedSseHub {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            topics: Arc::default(),
            capacity,
        }
    }
    pub fn subscribe(&self, topic: impl Into<String>) -> BoundedSseReceiver {
        let mut topics = self.topics.lock().expect("SSE topic lock poisoned");
        let sender = topics
            .entry(topic.into())
            .or_insert_with(|| broadcast::channel(self.capacity).0);
        BoundedSseReceiver {
            receiver: sender.subscribe(),
        }
    }
    pub fn publish(&self, topic: &str, message: SseMessage) {
        if let Some(sender) = self
            .topics
            .lock()
            .expect("SSE topic lock poisoned")
            .get(topic)
        {
            let _ = sender.send(message);
        }
    }
    pub fn close(&self) {
        self.topics.lock().expect("SSE topic lock poisoned").clear();
    }
    pub fn remove_topic(&self, topic: &str) {
        self.topics
            .lock()
            .expect("SSE topic lock poisoned")
            .remove(topic);
    }
}
impl BoundedSseReceiver {
    pub async fn recv(&mut self) -> Option<SseMessage> {
        match self.receiver.recv().await {
            Ok(message) => Some(message),
            Err(broadcast::error::RecvError::Lagged(count)) => {
                Some(SseMessage::new("gap", format!("{{\"dropped\":{count}}}")))
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    }
}
