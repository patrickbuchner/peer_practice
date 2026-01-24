use crate::current::chat::ChatId;
use crate::v2026_01_11::chat::ChatMessageFromServer;
use crate::v2026_01_11::post::{Post, PostId};
use crate::v2026_01_11::user::UserId;
use crate::v2026_01_11::user::display_user::UserDisplay;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClient {
    MessageNotYetKnown,
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
    ChatDoesNotExistForPost(PostId),
    ChatDoesNotExist(ChatId),
    Chat(ChatId, Vec<ChatMessageFromServer>),
    MessageSent(ChatMessageFromServer),
}

pub mod transformations_v2025_10_14 {
    use crate::v2026_01_11::messages::server_to_client::{PostAction, ServerToClient, UserAction};

    impl From<ServerToClient> for crate::v2025_10_14::messages::ServerToClient {
        fn from(value: ServerToClient) -> Self {
            match value {
                ServerToClient::User(UserAction::User(id, display)) => Self::User(id, display),
                ServerToClient::User(UserAction::YouAre(id)) => Self::YouAre(id),
                ServerToClient::Post(PostAction::Post(id, post)) => Self::Post(id, post),
                ServerToClient::Post(PostAction::RemovedPost(id)) => Self::RemovedPost(id),
                ServerToClient::Chat(_) => Self::MessageNotYetKnown,
                ServerToClient::MessageNotYetKnown => Self::MessageNotYetKnown,
            }
        }
    }

    impl From<crate::v2025_10_14::messages::ServerToClient> for ServerToClient {
        fn from(value: crate::v2025_10_14::messages::ServerToClient) -> Self {
            use crate::v2025_10_14::messages::ServerToClient as Old;
            match value {
                Old::User(id, display) => Self::User(UserAction::User(id, display)),
                Old::Post(id, post) => Self::Post(PostAction::Post(id, post)),
                Old::RemovedPost(id) => Self::Post(PostAction::RemovedPost(id)),
                Old::YouAre(id) => Self::User(UserAction::YouAre(id)),
                Old::MessageNotYetKnown => Self::MessageNotYetKnown,
            }
        }
    }
}
