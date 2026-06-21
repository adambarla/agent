use std::vec::Vec;

#[derive(Clone, Debug)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Clone, Debug)]
pub enum EventPayload {
    Message {
        role: MessageRole,
        content: String,
        tool_calls: Vec<ToolCall>,
    },
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: EventPayload,
}

impl Event {
    pub fn new(payload: EventPayload) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            payload,
        }
    }
}

pub trait Session {
    fn append(&mut self, event: &Event);
    fn get_all_events(&self) -> &Vec<Event>;
}

#[derive(Default)]
pub struct InMemorySession {
    events: Vec<Event>,
}

impl Session for InMemorySession {
    fn append(&mut self, event: &Event) {
        self.events.push(event.clone());
    }

    fn get_all_events(&self) -> &Vec<Event> {
        &self.events
    }
}
