#[derive(Clone, Debug)]
pub struct SseMessage {
    pub event: String,
    pub data: String,
}

impl SseMessage {
    pub fn new(event: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            data: data.into(),
        }
    }
}
