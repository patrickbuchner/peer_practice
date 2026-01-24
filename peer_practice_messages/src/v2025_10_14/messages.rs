use super::post::{Post, PostId};
use super::user::UserId;
use super::user::display_user::UserDisplay;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerToClient {
    MessageNotYetKnown,
    User(UserId, UserDisplay),
    Post(PostId, Post),
    RemovedPost(PostId),
    YouAre(UserId),
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    MessageNotYetKnown,
    Hello,
    GetUser(UserId),
    UpdateUser(UserDisplay),
    GetPosts,
    Join(PostId),
    Leave(PostId),
    UpdatePost(PostId, Post),
    NewPost(Post),
    DeletePost(PostId),
}
