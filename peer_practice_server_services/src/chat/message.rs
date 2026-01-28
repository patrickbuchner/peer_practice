use peer_practice_messages::current::chat::{
    ChatId, ChatMessage, ChatMessageFromServer, ChatMessageKind,
};
use peer_practice_messages::current::user::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub sender: UserId,
    pub kind: ChatMessageKind,
    pub chat_id: ChatId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<Message> for ChatMessageFromServer {
    fn from(msg: Message) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind,
            chat_id: msg.chat_id,
            timestamp: msg.timestamp,
        }
    }
}

impl From<&Message> for ChatMessageFromServer {
    fn from(msg: &Message) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind.clone(),
            chat_id: msg.chat_id,
            timestamp: msg.timestamp,
        }
    }
}

impl From<ChatMessageFromServer> for Message {
    fn from(msg: ChatMessageFromServer) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind,
            chat_id: msg.chat_id,
            timestamp: msg.timestamp,
        }
    }
}

impl From<Message> for ChatMessage {
    fn from(msg: Message) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind,
            chat_id: msg.chat_id,
        }
    }
}

impl From<&Message> for ChatMessage {
    fn from(msg: &Message) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind.clone(),
            chat_id: msg.chat_id,
        }
    }
}

impl From<ChatMessage> for Message {
    fn from(msg: ChatMessage) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind,
            chat_id: msg.chat_id,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl From<&ChatMessage> for Message {
    fn from(msg: &ChatMessage) -> Self {
        Self {
            sender: msg.sender,
            kind: msg.kind.clone(),
            chat_id: msg.chat_id,
            timestamp: chrono::Utc::now(),
        }
    }
}
