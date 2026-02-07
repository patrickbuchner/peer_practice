use crate::v2026_02_07::chat::ChatId;
use crate::v2026_02_07::chat::ChatMessageFromServer;
use crate::v2026_02_07::post::{Post, PostId};
use crate::v2026_02_07::user::UserId;
use crate::v2026_02_07::user::display_user::UserDisplay;
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
    Chat(ChatId, PostId, Vec<ChatMessageFromServer>),
    MessageSent(ChatMessageFromServer),
}

pub mod transformations_v2026_01_11 {
    use crate::v2026_02_07::messages::server_to_client::{PostAction, ServerToClient, UserAction};

    impl From<ServerToClient> for crate::v2026_01_11::messages::ServerToClient {
        fn from(value: ServerToClient) -> Self {
            match value {
                ServerToClient::MessageNotYetKnown => Self::MessageNotYetKnown,

                ServerToClient::User(UserAction::User(id, display)) => Self::User(
                    crate::v2026_01_11::messages::server_to_client::UserAction::User(id, display),
                ),
                ServerToClient::User(UserAction::YouAre(id)) => Self::User(
                    crate::v2026_01_11::messages::server_to_client::UserAction::YouAre(id),
                ),

                ServerToClient::Post(PostAction::Post(id, post)) => Self::Post(
                    crate::v2026_01_11::messages::server_to_client::PostAction::Post(id, post),
                ),
                ServerToClient::Post(PostAction::RemovedPost(id)) => Self::Post(
                    crate::v2026_01_11::messages::server_to_client::PostAction::RemovedPost(id),
                ),

                ServerToClient::Chat(_) => Self::MessageNotYetKnown,
            }
        }
    }

    impl From<crate::v2026_01_11::messages::ServerToClient> for ServerToClient {
        fn from(value: crate::v2026_01_11::messages::ServerToClient) -> Self {
            use crate::v2026_01_11::messages::ServerToClient as Old;

            match value {
                Old::MessageNotYetKnown => Self::MessageNotYetKnown,

                Old::User(crate::v2026_01_11::messages::server_to_client::UserAction::User(
                    id,
                    display,
                )) => Self::User(UserAction::User(id, display)),
                Old::User(crate::v2026_01_11::messages::server_to_client::UserAction::YouAre(
                    id,
                )) => Self::User(UserAction::YouAre(id)),

                Old::Post(crate::v2026_01_11::messages::server_to_client::PostAction::Post(
                    id,
                    post,
                )) => Self::Post(PostAction::Post(id, post)),
                Old::Post(
                    crate::v2026_01_11::messages::server_to_client::PostAction::RemovedPost(id),
                ) => Self::Post(PostAction::RemovedPost(id)),

                Old::Chat(_) => Self::MessageNotYetKnown,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2026_01_11::messages::ServerToClient as Old;

    #[test]
    fn chat_actions_map_to_legacy_unknown() {
        let msg = ServerToClient::Chat(ChatAction::ChatDoesNotExistForPost(PostId::new()));
        let legacy: Old = msg.into();
        assert!(matches!(legacy, Old::MessageNotYetKnown));
    }
}
