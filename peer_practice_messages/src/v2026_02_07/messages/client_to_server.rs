use crate::v2026_02_07::chat::{ChatId, ChatMessage};
use crate::v2026_02_07::post::{Post, PostId};
use crate::v2026_02_07::sessions::{SessionId, SessionInformation};
use crate::v2026_02_07::user::UserId;
use crate::v2026_02_07::user::display_user::UserDisplay;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum ClientToServer {
    Hello,
    MessageNotYetKnown,
    MessageRemoved,
    User(UserAction),
    Post(PostAction),
    Chat(ChatAction),
    Session(SessionAction),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum UserAction {
    Get(UserId),
    Update(UserDisplay),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum PostAction {
    GetPosts,
    Join(PostId),
    Leave(PostId),
    UpdatePost(PostId, Post),
    NewPost(Post),
    DeletePost(PostId),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum ChatAction {
    GetChatFor(PostId),
    GetChat(ChatId),
    SendMessage(ChatMessage),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum SessionAction {
    GetSessions,
    GetThisSession,
    UpdateSession(SessionInformation),
    LogOutSession(SessionId),
    LogOutAllSessions,
}

mod transformations_v2026_01_11 {
    use crate::v2026_02_07::messages::client_to_server::{
        ChatAction, ClientToServer, PostAction, UserAction,
    };

    impl From<ClientToServer> for crate::v2026_01_11::messages::ClientToServer {
        fn from(value: ClientToServer) -> Self {
            match value {
                ClientToServer::Hello => Self::Hello,
                ClientToServer::MessageNotYetKnown => Self::MessageNotYetKnown,
                ClientToServer::MessageRemoved => Self::MessageNotYetKnown,

                ClientToServer::User(UserAction::Get(id)) => {
                    Self::User(crate::v2026_01_11::messages::client_to_server::UserAction::Get(id))
                }
                ClientToServer::User(UserAction::Update(user)) => Self::User(
                    crate::v2026_01_11::messages::client_to_server::UserAction::Update(user),
                ),

                ClientToServer::Post(PostAction::GetPosts) => {
                    Self::Post(crate::v2026_01_11::messages::client_to_server::PostAction::GetPosts)
                }
                ClientToServer::Post(PostAction::Join(id)) => {
                    Self::Post(crate::v2026_01_11::messages::client_to_server::PostAction::Join(id))
                }
                ClientToServer::Post(PostAction::Leave(id)) => Self::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::Leave(id),
                ),
                ClientToServer::Post(PostAction::UpdatePost(id, post)) => Self::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::UpdatePost(
                        id, post,
                    ),
                ),
                ClientToServer::Post(PostAction::NewPost(post)) => Self::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::NewPost(post),
                ),
                ClientToServer::Post(PostAction::DeletePost(id)) => Self::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::DeletePost(id),
                ),

                ClientToServer::Chat(ChatAction::GetChatFor(id)) => Self::Chat(
                    crate::v2026_01_11::messages::client_to_server::ChatAction::GetChatFor(id),
                ),
                ClientToServer::Chat(ChatAction::GetChat(id)) => Self::Chat(
                    crate::v2026_01_11::messages::client_to_server::ChatAction::GetChat(id),
                ),
                ClientToServer::Chat(ChatAction::SendMessage(msg)) => Self::Chat(
                    crate::v2026_01_11::messages::client_to_server::ChatAction::SendMessage(msg),
                ),
                ClientToServer::Session(_) => Self::MessageNotYetKnown,
            }
        }
    }

    impl From<crate::v2026_01_11::messages::ClientToServer> for ClientToServer {
        fn from(value: crate::v2026_01_11::messages::ClientToServer) -> Self {
            use crate::v2026_01_11::messages::ClientToServer as Old;

            match value {
                Old::Hello => Self::Hello,
                Old::MessageNotYetKnown => Self::MessageNotYetKnown,

                Old::User(crate::v2026_01_11::messages::client_to_server::UserAction::Get(id)) => {
                    Self::User(UserAction::Get(id))
                }
                Old::User(crate::v2026_01_11::messages::client_to_server::UserAction::Update(
                    user,
                )) => Self::User(UserAction::Update(user)),

                Old::Post(crate::v2026_01_11::messages::client_to_server::PostAction::GetPosts) => {
                    Self::Post(PostAction::GetPosts)
                }
                Old::Post(crate::v2026_01_11::messages::client_to_server::PostAction::Join(id)) => {
                    Self::Post(PostAction::Join(id))
                }
                Old::Post(crate::v2026_01_11::messages::client_to_server::PostAction::Leave(
                    id,
                )) => Self::Post(PostAction::Leave(id)),
                Old::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::UpdatePost(
                        id,
                        post,
                    ),
                ) => Self::Post(PostAction::UpdatePost(id, post)),
                Old::Post(crate::v2026_01_11::messages::client_to_server::PostAction::NewPost(
                    post,
                )) => Self::Post(PostAction::NewPost(post)),
                Old::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::DeletePost(id),
                ) => Self::Post(PostAction::DeletePost(id)),
                Old::Post(
                    crate::v2026_01_11::messages::client_to_server::PostAction::GetPostMessages(_),
                ) => Self::MessageRemoved,

                Old::Chat(
                    crate::v2026_01_11::messages::client_to_server::ChatAction::GetChatFor(id),
                ) => Self::Chat(ChatAction::GetChatFor(id)),
                Old::Chat(crate::v2026_01_11::messages::client_to_server::ChatAction::GetChat(
                    id,
                )) => Self::Chat(ChatAction::GetChat(id)),
                Old::Chat(
                    crate::v2026_01_11::messages::client_to_server::ChatAction::SendMessage(msg),
                ) => Self::Chat(ChatAction::SendMessage(msg)),
            }
        }
    }
}
