use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use eyre::Context;
use jsonwebtoken::{DecodingKey, Validation, decode};
use tokio::sync::oneshot;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::handler::claims::Claims;
use crate::handler::client_communication::handle_websocket_message;
use peer_practice_messages::current::messages::{ClientToServer, ServerToClient};
use peer_practice_messages::current::user::UserId;
use peer_practice_messages::{Envelope, EnvelopeHeader, Version};
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    let access_token = match jar.get("access_token") {
        Some(cookie) => cookie.value().to_string(),
        None => return (StatusCode::UNAUTHORIZED, "No access token").into_response(),
    };

    let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_ref());
    match decode::<Claims>(&access_token, &decoding_key, &Validation::default()) {
        Ok(token_data) => {
            info!(
                "User '{:?}' connected via WebSocket",
                token_data.claims.user_id
            );
            ws.on_upgrade(move |socket| handle_socket(socket, token_data.claims.user_id, state))
        }
        Err(e) => {
            error!("{e}");
            (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
        }
    }
}

async fn handle_socket(mut socket: WebSocket, user_id: UserId, state: AppState) {
    let (tx, rx) = oneshot::channel();
    let _ = state
        .ws_hub
        .send(WsHubMsg::Join {
            user_id,
            respond_to: tx,
        })
        .await;

    let (connection_handle, mut hub_rx) = match rx.await {
        Ok(result) => result,
        Err(e) => {
            error!(
                "Failed to register WS connection in hub for {:?}: {}",
                user_id, e
            );
            return;
        }
    };

    let connection_id = connection_handle.id();
    let mut client_version = Version::default();

    loop {
        tokio::select! {
            maybe_msg = hub_rx.recv() => {
                match maybe_msg {
                    Some(server_msg) => {
                        let res = send_message(&mut socket, server_msg, client_version).await;
                        match res {
                            Ok(()) => {}
                            Err(e) => {
                                error!("Failed to send message to client: {}", e);
                                break;
                            }
                        }
                    }
                    None => {
                        // Hub closed our channel; end the connection
                        break;
                    }
                }
            }

            maybe_ws = socket.recv() => {
                match maybe_ws {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(header) = serde_json::from_str::<EnvelopeHeader>(&text) {
                            client_version = header.version;

                        }

                        if let Err(report) = consume_message(connection_id, &state, user_id, &text, client_version).await{
                            error!("Failed to consume message from client: {report:?}");
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Ok(_)) => {
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for {:?}: {}", user_id, e);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}

async fn send_message(
    socket: &mut WebSocket,
    msg: ServerToClient,
    version: Version,
) -> Result<(), String> {
    let text = serialize_server_message(&msg, version)?;

    info!("Sending message to client: {}", text);

    if socket.send(Message::Text(text.into())).await.is_err() {
        Err("Socket has been disconnected".into())
    } else {
        Ok(())
    }
}

fn serialize_server_message(msg: &ServerToClient, version: Version) -> Result<String, String> {
    let text = match version {
        Version::V2026_01_11 => serde_json::to_string(&Envelope {
            version: Version::V2026_01_11,
            data: msg.clone(),
        }),
        Version::V2025_10_14 => serde_json::to_string(&Envelope::<
            peer_practice_messages::v2025_10_14::messages::ServerToClient,
        > {
            version: Version::V2025_10_14,
            data: msg.clone().into(),
        }),
    };

    text.map_err(|e| format!("Failed to serialize message: {}", e))
}

async fn consume_message(
    connection_id: ConnectionId,
    state: &AppState,
    user_id: UserId,
    text: &Utf8Bytes,
    version: Version,
) -> eyre::Result<()> {
    info!("Received message from {:?}: {}", user_id, text);
    let data = parse_client_message(version, text)?;
    handle_websocket_message(connection_id, state, user_id, data)
        .await
        .wrap_err("Failed to handle websocket message")?;
    Ok(())
}

fn parse_client_message(version: Version, text: &str) -> eyre::Result<ClientToServer> {
    match version {
        Version::V2026_01_11 => Ok(serde_json::from_str::<Envelope<ClientToServer>>(text)
            .wrap_err("Failed to parse Envelope")?
            .data),
        Version::V2025_10_14 => {
            let legacy = serde_json::from_str::<Envelope<
                peer_practice_messages::v2025_10_14::messages::ClientToServer,
            >>(text)
            .wrap_err("Failed to parse Envelope")?;
            Ok(legacy.data.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peer_practice_messages::current::messages::client_to_server::PostAction;
    use peer_practice_messages::current::messages::client_to_server::{
        ChatAction as ClientChatAction, UserAction as ClientUserAction,
    };
    use peer_practice_messages::current::messages::server_to_client::{
        ChatAction as ServerChatAction, PostAction as ServerPostAction,
        UserAction as ServerUserAction,
    };
    use peer_practice_messages::v2025_10_14::messages::ClientToServer as OldClientToServer;
    use peer_practice_messages::v2025_10_14::messages::ServerToClient as OldServerToClient;
    use peer_practice_messages::v2025_10_14::post::{Post, PostId, Topics};
    use peer_practice_messages::v2025_10_14::user::UserId;
    use peer_practice_messages::v2025_10_14::user::display_user::UserDisplay;
    use chrono::TimeZone;
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};
    use serde_json::Value;
    use std::collections::HashSet;

    #[test]
    fn serialize_server_message_sets_version() {
        let msg = ServerToClient::MessageNotYetKnown;
        let text = serialize_server_message(&msg, Version::V2026_01_11).unwrap();
        let envelope = serde_json::from_str::<Envelope<ServerToClient>>(&text).unwrap();
        assert!(matches!(envelope.version, Version::V2026_01_11));

        let text = serialize_server_message(&msg, Version::V2025_10_14).unwrap();
        let envelope = serde_json::from_str::<Envelope<
            peer_practice_messages::v2025_10_14::messages::ServerToClient,
        >>(&text)
        .unwrap();
        assert!(matches!(envelope.version, Version::V2025_10_14));
    }

    #[test]
    fn parse_client_message_accepts_known_versions() {
        let text = serde_json::to_string(&Envelope {
            version: Version::V2026_01_11,
            data: ClientToServer::Hello,
        })
        .unwrap();
        let parsed = parse_client_message(Version::V2026_01_11, &text).unwrap();
        assert!(matches!(parsed, ClientToServer::Hello));

        let text = serde_json::to_string(&Envelope {
            version: Version::V2025_10_14,
            data: OldClientToServer::GetPosts,
        })
        .unwrap();
        let parsed = parse_client_message(Version::V2025_10_14, &text).unwrap();
        assert!(matches!(
            parsed,
            ClientToServer::Post(PostAction::GetPosts)
        ));
    }

    #[test]
    fn parse_client_message_rejects_invalid_json() {
        let err = parse_client_message(Version::V2026_01_11, "not-json");
        assert!(err.is_err());
    }

    #[test]
    fn parse_client_message_handles_arbitrary_inputs() {
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..256 {
            let mut bytes = vec![0u8; (rng.next_u32() % 128) as usize];
            rng.fill_bytes(&mut bytes);
            let input = String::from_utf8_lossy(&bytes).to_string();
            if serde_json::from_str::<Envelope<Value>>(&input).is_err() {
                assert!(parse_client_message(Version::V2026_01_11, &input).is_err());
                assert!(parse_client_message(Version::V2025_10_14, &input).is_err());
            }
        }
    }

    #[test]
    fn client_messages_roundtrip_current_version() {
        for msg in sample_current_client_messages() {
            let expected = client_kind(&msg);
            let text = serde_json::to_string(&Envelope {
                version: Version::V2026_01_11,
                data: msg,
            })
            .unwrap();
            let parsed = parse_client_message(Version::V2026_01_11, &text).unwrap();
            assert_eq!(expected, client_kind(&parsed));
        }
    }

    #[test]
    fn client_messages_roundtrip_legacy_version() {
        for msg in sample_legacy_client_messages() {
            let expected = legacy_client_kind(&msg);
            let text = serde_json::to_string(&Envelope {
                version: Version::V2025_10_14,
                data: msg,
            })
            .unwrap();
            let parsed = parse_client_message(Version::V2025_10_14, &text).unwrap();
            assert_eq!(expected, client_kind(&parsed));
        }
    }

    #[test]
    fn server_messages_roundtrip_current_version() {
        for msg in sample_current_server_messages() {
            let text = serialize_server_message(&msg, Version::V2026_01_11).unwrap();
            let envelope = serde_json::from_str::<Envelope<ServerToClient>>(&text).unwrap();
            assert_eq!(server_kind(&msg), server_kind(&envelope.data));
        }
    }

    #[test]
    fn server_messages_roundtrip_legacy_version() {
        for msg in sample_current_server_messages() {
            let text = serialize_server_message(&msg, Version::V2025_10_14).unwrap();
            let envelope = serde_json::from_str::<Envelope<OldServerToClient>>(&text).unwrap();
            assert_eq!(
                legacy_server_kind_from_current(&msg),
                legacy_server_kind(&envelope.data)
            );
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum ClientKind {
        Hello,
        MessageNotYetKnown,
        UserGet,
        UserUpdate,
        PostGetPosts,
        PostJoin,
        PostLeave,
        PostUpdatePost,
        PostNewPost,
        PostDeletePost,
        PostGetPostMessages,
        ChatGetChatFor,
        ChatGetChat,
        ChatSendMessage,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum ServerKind {
        MessageNotYetKnown,
        User,
        YouAre,
        Post,
        RemovedPost,
        ChatDoesNotExistForPost,
        ChatDoesNotExist,
        Chat,
        MessageSent,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum LegacyServerKind {
        MessageNotYetKnown,
        User,
        YouAre,
        Post,
        RemovedPost,
    }

    fn client_kind(msg: &ClientToServer) -> ClientKind {
        match msg {
            ClientToServer::Hello => ClientKind::Hello,
            ClientToServer::MessageNotYetKnown => ClientKind::MessageNotYetKnown,
            ClientToServer::User(ClientUserAction::Get(_)) => ClientKind::UserGet,
            ClientToServer::User(ClientUserAction::Update(_)) => ClientKind::UserUpdate,
            ClientToServer::Post(PostAction::GetPosts) => ClientKind::PostGetPosts,
            ClientToServer::Post(PostAction::Join(_)) => ClientKind::PostJoin,
            ClientToServer::Post(PostAction::Leave(_)) => ClientKind::PostLeave,
            ClientToServer::Post(PostAction::UpdatePost(_, _)) => ClientKind::PostUpdatePost,
            ClientToServer::Post(PostAction::NewPost(_)) => ClientKind::PostNewPost,
            ClientToServer::Post(PostAction::DeletePost(_)) => ClientKind::PostDeletePost,
            ClientToServer::Post(PostAction::GetPostMessages(_)) => ClientKind::PostGetPostMessages,
            ClientToServer::Chat(ClientChatAction::GetChatFor(_)) => ClientKind::ChatGetChatFor,
            ClientToServer::Chat(ClientChatAction::GetChat(_)) => ClientKind::ChatGetChat,
            ClientToServer::Chat(ClientChatAction::SendMessage(_)) => ClientKind::ChatSendMessage,
        }
    }

    fn legacy_client_kind(msg: &OldClientToServer) -> ClientKind {
        match msg {
            OldClientToServer::Hello => ClientKind::Hello,
            OldClientToServer::MessageNotYetKnown => ClientKind::MessageNotYetKnown,
            OldClientToServer::GetUser(_) => ClientKind::UserGet,
            OldClientToServer::UpdateUser(_) => ClientKind::UserUpdate,
            OldClientToServer::GetPosts => ClientKind::PostGetPosts,
            OldClientToServer::Join(_) => ClientKind::PostJoin,
            OldClientToServer::Leave(_) => ClientKind::PostLeave,
            OldClientToServer::UpdatePost(_, _) => ClientKind::PostUpdatePost,
            OldClientToServer::NewPost(_) => ClientKind::PostNewPost,
            OldClientToServer::DeletePost(_) => ClientKind::PostDeletePost,
        }
    }

    fn server_kind(msg: &ServerToClient) -> ServerKind {
        match msg {
            ServerToClient::MessageNotYetKnown => ServerKind::MessageNotYetKnown,
            ServerToClient::User(ServerUserAction::User(_, _)) => ServerKind::User,
            ServerToClient::User(ServerUserAction::YouAre(_)) => ServerKind::YouAre,
            ServerToClient::Post(ServerPostAction::Post(_, _)) => ServerKind::Post,
            ServerToClient::Post(ServerPostAction::RemovedPost(_)) => ServerKind::RemovedPost,
            ServerToClient::Chat(ServerChatAction::ChatDoesNotExistForPost(_)) => {
                ServerKind::ChatDoesNotExistForPost
            }
            ServerToClient::Chat(ServerChatAction::ChatDoesNotExist(_)) => {
                ServerKind::ChatDoesNotExist
            }
            ServerToClient::Chat(ServerChatAction::Chat(_, _)) => ServerKind::Chat,
            ServerToClient::Chat(ServerChatAction::MessageSent(_)) => ServerKind::MessageSent,
        }
    }

    fn legacy_server_kind(msg: &OldServerToClient) -> LegacyServerKind {
        match msg {
            OldServerToClient::MessageNotYetKnown => LegacyServerKind::MessageNotYetKnown,
            OldServerToClient::User(_, _) => LegacyServerKind::User,
            OldServerToClient::YouAre(_) => LegacyServerKind::YouAre,
            OldServerToClient::Post(_, _) => LegacyServerKind::Post,
            OldServerToClient::RemovedPost(_) => LegacyServerKind::RemovedPost,
        }
    }

    fn legacy_server_kind_from_current(msg: &ServerToClient) -> LegacyServerKind {
        match msg {
            ServerToClient::MessageNotYetKnown => LegacyServerKind::MessageNotYetKnown,
            ServerToClient::User(ServerUserAction::User(_, _)) => LegacyServerKind::User,
            ServerToClient::User(ServerUserAction::YouAre(_)) => LegacyServerKind::YouAre,
            ServerToClient::Post(ServerPostAction::Post(_, _)) => LegacyServerKind::Post,
            ServerToClient::Post(ServerPostAction::RemovedPost(_)) => LegacyServerKind::RemovedPost,
            ServerToClient::Chat(_) => LegacyServerKind::MessageNotYetKnown,
        }
    }

    fn sample_user_id() -> UserId {
        UserId::test()
    }

    fn sample_user_display() -> UserDisplay {
        UserDisplay {
            display_name: Some("Test User".to_string()),
            id: sample_user_id(),
        }
    }

    fn sample_post_id() -> PostId {
        PostId::NULL
    }

    fn sample_post() -> Post {
        let mut partaking_users = HashSet::new();
        partaking_users.insert(sample_user_id());
        Post {
            title: Topics::Basics,
            content: "Example".to_string(),
            level: peer_practice_messages::v2025_10_14::level::Level::Beginner1,
            owner: sample_user_id(),
            date: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            partaking_users,
        }
    }

    fn sample_chat_id() -> peer_practice_messages::current::chat::ChatId {
        peer_practice_messages::current::chat::ChatId::new()
    }

    fn sample_chat_message(
        chat_id: peer_practice_messages::current::chat::ChatId,
    ) -> peer_practice_messages::current::chat::ChatMessage {
        peer_practice_messages::current::chat::ChatMessage {
            sender: sample_user_id(),
            message: "Hello".to_string(),
            chat_id,
        }
    }

    fn sample_chat_message_from_server(
        chat_id: peer_practice_messages::current::chat::ChatId,
    ) -> peer_practice_messages::current::chat::ChatMessageFromServer {
        peer_practice_messages::current::chat::ChatMessageFromServer {
            sender: sample_user_id(),
            message: "Hi".to_string(),
            chat_id,
            timestamp: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 1, 0, 0).unwrap(),
        }
    }

    fn sample_current_client_messages() -> Vec<ClientToServer> {
        let chat_id = sample_chat_id();
        vec![
            ClientToServer::Hello,
            ClientToServer::MessageNotYetKnown,
            ClientToServer::User(ClientUserAction::Get(sample_user_id())),
            ClientToServer::User(ClientUserAction::Update(sample_user_display())),
            ClientToServer::Post(PostAction::GetPosts),
            ClientToServer::Post(PostAction::Join(sample_post_id())),
            ClientToServer::Post(PostAction::Leave(sample_post_id())),
            ClientToServer::Post(PostAction::UpdatePost(sample_post_id(), sample_post())),
            ClientToServer::Post(PostAction::NewPost(sample_post())),
            ClientToServer::Post(PostAction::DeletePost(sample_post_id())),
            ClientToServer::Post(PostAction::GetPostMessages(sample_post_id())),
            ClientToServer::Chat(ClientChatAction::GetChatFor(sample_post_id())),
            ClientToServer::Chat(ClientChatAction::GetChat(chat_id)),
            ClientToServer::Chat(ClientChatAction::SendMessage(sample_chat_message(chat_id))),
        ]
    }

    fn sample_legacy_client_messages() -> Vec<OldClientToServer> {
        vec![
            OldClientToServer::Hello,
            OldClientToServer::MessageNotYetKnown,
            OldClientToServer::GetUser(sample_user_id()),
            OldClientToServer::UpdateUser(sample_user_display()),
            OldClientToServer::GetPosts,
            OldClientToServer::Join(sample_post_id()),
            OldClientToServer::Leave(sample_post_id()),
            OldClientToServer::UpdatePost(sample_post_id(), sample_post()),
            OldClientToServer::NewPost(sample_post()),
            OldClientToServer::DeletePost(sample_post_id()),
        ]
    }

    fn sample_current_server_messages() -> Vec<ServerToClient> {
        let chat_id = sample_chat_id();
        let chat_message = sample_chat_message_from_server(chat_id);
        vec![
            ServerToClient::MessageNotYetKnown,
            ServerToClient::User(ServerUserAction::User(
                sample_user_id(),
                sample_user_display(),
            )),
            ServerToClient::User(ServerUserAction::YouAre(sample_user_id())),
            ServerToClient::Post(ServerPostAction::Post(sample_post_id(), sample_post())),
            ServerToClient::Post(ServerPostAction::RemovedPost(sample_post_id())),
            ServerToClient::Chat(ServerChatAction::ChatDoesNotExistForPost(
                sample_post_id(),
            )),
            ServerToClient::Chat(ServerChatAction::ChatDoesNotExist(chat_id)),
            ServerToClient::Chat(ServerChatAction::Chat(chat_id, vec![chat_message.clone()])),
            ServerToClient::Chat(ServerChatAction::MessageSent(chat_message)),
        ]
    }
}
