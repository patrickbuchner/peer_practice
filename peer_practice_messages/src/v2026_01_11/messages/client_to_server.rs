use crate::v2026_01_11::chat::{ChatId, ChatMessage};
use crate::v2026_01_11::post::{Post, PostId};
use crate::v2026_01_11::user::UserId;
use crate::v2026_01_11::user::display_user::UserDisplay;
use serde::{Deserialize, Serialize};

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientToServer {
    Hello,
    MessageNotYetKnown,
    User(UserAction),
    Post(PostAction),
    Chat(ChatAction),
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub enum UserAction {
    Get(UserId),
    Update(UserDisplay),
}

#[cfg_attr(test, derive(Clone))]
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

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub enum ChatAction {
    GetChatFor(PostId),
    GetChat(ChatId),
    SendMessage(ChatMessage),
}

mod transformations_v2025_10_14 {
    use crate::v2026_01_11::messages::client_to_server::{ClientToServer, PostAction, UserAction};

    impl From<ClientToServer> for crate::v2025_10_14::messages::ClientToServer {
        fn from(value: ClientToServer) -> Self {
            match value {
                ClientToServer::Hello => Self::Hello,
                ClientToServer::User(UserAction::Get(id)) => Self::GetUser(id),
                ClientToServer::User(UserAction::Update(user)) => Self::UpdateUser(user),
                ClientToServer::Post(PostAction::GetPosts) => Self::GetPosts,
                ClientToServer::Post(PostAction::Join(id)) => Self::Join(id),
                ClientToServer::Post(PostAction::Leave(id)) => Self::Leave(id),
                ClientToServer::Post(PostAction::UpdatePost(id, post)) => {
                    Self::UpdatePost(id, post)
                }
                ClientToServer::Post(PostAction::NewPost(post)) => Self::NewPost(post),
                ClientToServer::Post(PostAction::DeletePost(id)) => Self::DeletePost(id),
                ClientToServer::Post(PostAction::GetPostMessages(_)) => Self::MessageNotYetKnown,
                ClientToServer::MessageNotYetKnown => Self::MessageNotYetKnown,
                ClientToServer::Chat(_) => Self::MessageNotYetKnown,
            }
        }
    }

    impl From<crate::v2025_10_14::messages::ClientToServer> for ClientToServer {
        fn from(value: crate::v2025_10_14::messages::ClientToServer) -> Self {
            use crate::v2025_10_14::messages::ClientToServer as Old;
            match value {
                Old::Hello => Self::Hello,
                Old::GetUser(id) => Self::User(UserAction::Get(id)),
                Old::UpdateUser(user) => Self::User(UserAction::Update(user)),
                Old::GetPosts => Self::Post(PostAction::GetPosts),
                Old::Join(id) => Self::Post(PostAction::Join(id)),
                Old::Leave(id) => Self::Post(PostAction::Leave(id)),
                Old::UpdatePost(id, post) => Self::Post(PostAction::UpdatePost(id, post)),
                Old::NewPost(post) => Self::Post(PostAction::NewPost(post)),
                Old::DeletePost(id) => Self::Post(PostAction::DeletePost(id)),
                Old::MessageNotYetKnown => Self::MessageNotYetKnown,
            }
        }
    }
}
