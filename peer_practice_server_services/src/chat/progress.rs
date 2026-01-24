use crate::chat::message::Message;
use peer_practice_messages::current::chat::ChatId;
use peer_practice_messages::current::post::PostId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub chat_id: ChatId,
    pub post_id: PostId,
    pub content: Vec<Message>,
}
