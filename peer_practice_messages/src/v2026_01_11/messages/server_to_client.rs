use crate::v2026_01_11::post::{Post, PostId};
use crate::v2026_01_11::user::UserId;
use crate::v2026_01_11::user::display_user::UserDisplay;
use crate::v2026_01_11::chat::ChatMessageFromServer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClient {
    User(UserAction),
    Post(PostAction),
    Chat(ChatAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAction {
    User(UserId, UserDisplay),
    YouAre(UserId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostAction {
    Post(PostId, Post),
    RemovedPost(PostId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatAction {
    Chat(Vec<ChatMessageFromServer>),
    MessageSent(ChatMessageFromServer),
}
