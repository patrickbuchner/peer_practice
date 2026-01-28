use crate::v2026_01_11::user::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ChatMessageKind {
    Text(String),
    Joined,
    Left,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChatMessage {
    pub sender: UserId,
    pub kind: ChatMessageKind,
    pub chat_id: ChatId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChatMessageFromServer {
    pub sender: UserId,
    pub kind: ChatMessageKind,
    pub chat_id: ChatId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChatId {
    id: Uuid,
}
impl Default for ChatId {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn from_id(id: Uuid) -> Self {
        Self { id }
    }
}
