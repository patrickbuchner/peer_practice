use crate::v2026_01_11::chat::{ChatId, ChatMessage};
use crate::v2026_01_11::post::{Post, PostId};
use crate::v2026_01_11::user::UserId;
use crate::v2026_01_11::user::display_user::UserDisplay;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    Hello,
    User(UserAction),
    Post(PostAction),
    Chat(ChatAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserAction {
    Get(UserId),
    Update(UserDisplay),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PostAction {
    GetPosts,
    Join(PostId),
    Leave(PostId),
    UpdatePost(PostId, Post),
    NewPost(Post),
    DeletePost(PostId),
    GetPostMessages(PostId),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChatAction {
    GetChatFor(PostId),
    GetChat(ChatId),
    SendMessage(ChatMessage),
}
