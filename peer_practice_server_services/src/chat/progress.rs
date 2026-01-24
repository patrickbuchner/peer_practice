use peer_practice_messages::current::chat::ChatId;
use peer_practice_messages::current::post::PostId;
use crate::chat::message::Message;

#[derive(Debug)]
pub struct Progress {
    pub chat_id: ChatId,
    pub post_id: PostId,
    pub content: Vec<Message>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}