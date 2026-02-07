use crate::current::sessions::SessionId;
use crate::v2026_02_07::chat::ChatId;
use crate::v2026_02_07::chat::ChatMessageFromServer;
use crate::v2026_02_07::post::{Post, PostId};
use crate::v2026_02_07::user::UserId;
use crate::v2026_02_07::user::display_user::UserDisplay;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ServerToClient {
    MessageNotYetKnown,
    User(UserAction),
    Post(PostAction),
    Chat(ChatAction),
    Session(SessionAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UserAction {
    User(UserId, UserDisplay),
    YouAre(UserId),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum PostAction {
    Post(PostId, Post),
    RemovedPost(PostId),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ChatAction {
    ChatDoesNotExistForPost(PostId),
    ChatDoesNotExist(ChatId),
    Chat(ChatId, PostId, Vec<ChatMessageFromServer>),
    MessageSent(ChatMessageFromServer),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum SessionAction {
    CurrentSession(SessionId),
    Sessions(Vec<SessionId>),
}

pub mod transformations_v2026_01_11 {
    use crate::current::messages::server_to_client::ChatAction;
    use crate::v2026_02_07::messages::server_to_client::{PostAction, ServerToClient, UserAction};

    use crate::v2026_01_11::messages::server_to_client as previous;
    impl From<ServerToClient> for crate::v2026_01_11::messages::ServerToClient {
        fn from(value: ServerToClient) -> Self {
            match value {
                ServerToClient::MessageNotYetKnown => Self::MessageNotYetKnown,

                ServerToClient::User(a) => {
                    let action = match a {
                        UserAction::User(id, display) => previous::UserAction::User(id, display),
                        UserAction::YouAre(id) => previous::UserAction::YouAre(id),
                    };

                    Self::User(action)
                }
                ServerToClient::Post(action) => {
                    let action = match action {
                        PostAction::Post(id, data) => previous::PostAction::Post(id, data),
                        PostAction::RemovedPost(id) => previous::PostAction::RemovedPost(id),
                    };
                    Self::Post(action)
                }
                ServerToClient::Chat(action) => {
                    let action = match action {
                        ChatAction::ChatDoesNotExistForPost(id) => {
                            previous::ChatAction::ChatDoesNotExistForPost(id)
                        }
                        ChatAction::ChatDoesNotExist(id) => {
                            previous::ChatAction::ChatDoesNotExist(id)
                        }
                        ChatAction::Chat(cid, pid, msgs) => {
                            previous::ChatAction::Chat(cid, pid, msgs)
                        }
                        ChatAction::MessageSent(msg) => previous::ChatAction::MessageSent(msg),
                    };
                    Self::Chat(action)
                }
                ServerToClient::Session(_) => Self::MessageNotYetKnown,
            }
        }
    }

    impl From<crate::v2026_01_11::messages::ServerToClient> for ServerToClient {
        fn from(value: crate::v2026_01_11::messages::ServerToClient) -> Self {
            use crate::v2026_01_11::messages::server_to_client as Old;

            match value {
                Old::ServerToClient::MessageNotYetKnown => Self::MessageNotYetKnown,

                Old::ServerToClient::User(action) => {
                    let action = match action {
                        Old::UserAction::User(id, display) => UserAction::User(id, display),
                        Old::UserAction::YouAre(id) => UserAction::YouAre(id),
                    };

                    Self::User(action)
                }

                Old::ServerToClient::Post(action) => {
                    let action = match action {
                        Old::PostAction::Post(id, post) => PostAction::Post(id, post),
                        Old::PostAction::RemovedPost(id) => PostAction::RemovedPost(id),
                    };

                    Self::Post(action)
                }

                Old::ServerToClient::Chat(action) => {
                    let action = match action {
                        Old::ChatAction::ChatDoesNotExistForPost(id) => {
                            ChatAction::ChatDoesNotExistForPost(id)
                        }
                        Old::ChatAction::ChatDoesNotExist(id) => ChatAction::ChatDoesNotExist(id),
                        Old::ChatAction::Chat(cid, pid, msgs) => ChatAction::Chat(cid, pid, msgs),
                        Old::ChatAction::MessageSent(msg) => ChatAction::MessageSent(msg),
                    };

                    Self::Chat(action)
                }
            }
        }
    }
}
