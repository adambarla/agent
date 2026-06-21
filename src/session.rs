use std::{collections::HashSet, vec::Vec};

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
    fn append(&mut self, event: &Event) -> bool;
    fn get_all_events(&self) -> &Vec<Event>;
}

#[derive(Default)]
pub struct InMemorySession {
    events: Vec<Event>,
    event_ids: HashSet<String>,
}

impl Session for InMemorySession {
    fn append(&mut self, event: &Event) -> bool {
        if self.event_ids.contains(&event.id) {
            return false;
        }
        self.events.push(event.clone());
        self.event_ids.insert(event.id.clone());
        true
    }

    fn get_all_events(&self) -> &Vec<Event> {
        &self.events
    }
}

#[macro_export]
macro_rules! test_session {
    ($name:ident, $session:expr) => {
        #[cfg(test)]
        mod $name {
            use super::*;
            use anyhow::Result;

            #[test]
            fn test_lifecycle() -> Result<()> {
                let mut session = $session;
                assert!(session.get_all_events().is_empty());
                let e1 = Event::new(EventPayload::Message {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    tool_calls: Vec::new(),
                });
                assert!(session.append(&e1.clone()));
                assert!(session.get_all_events().len() == 1);

                let was_appended = session.append(&e1);
                assert!(!was_appended);
                assert_eq!(session.get_all_events().len(), 1);
                Ok(())
            }
        }
    };
}

test_session!(in_memory_session, InMemorySession::default());
