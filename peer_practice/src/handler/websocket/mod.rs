use crate::app_state::AppState;
use crate::handler::claims::Claims;
use crate::handler::client_communication::handle_websocket_message;
use crate::handler::login::create_access_cookie;
use axum::Error;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use eyre::Context;
use jsonwebtoken::{DecodingKey, Validation, decode};
use peer_practice_messages::current::messages::{ClientToServer, ServerToClient};
use peer_practice_messages::current::user::UserId;
use peer_practice_messages::v2026_02_07::sessions::SessionId;
use peer_practice_messages::{Envelope, EnvelopeHeader, Version};
use peer_practice_server_services::active_sessions::{ActiveSessionsMsg, SessionState};
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use tokio::sync::oneshot;
use tracing::{error, info};

fn serialize_server_message(msg: &ServerToClient, version: Version) -> Result<String, String> {
    fn serialize_for_client_version(
        msg: &ServerToClient,
        version: Version,
    ) -> Result<String, serde_json::Error> {
        match version {
            Version::V2026_02_07 => serde_json::to_string(&Envelope {
                version: Version::V2026_02_07,
                data: msg.clone(),
            }),

            Version::V2026_01_11 => {
                let v01: peer_practice_messages::v2026_01_11::messages::ServerToClient =
                    msg.clone().into();

                serde_json::to_string(&Envelope {
                    version: Version::V2026_01_11,
                    data: v01,
                })
            }

            Version::V2025_10_14 => {
                let v01: peer_practice_messages::v2026_01_11::messages::ServerToClient =
                    msg.clone().into();
                let v10: peer_practice_messages::v2025_10_14::messages::ServerToClient = v01.into();

                serde_json::to_string(&Envelope {
                    version: Version::V2025_10_14,
                    data: v10,
                })
            }
        }
    }

    serialize_for_client_version(msg, version)
        .map_err(|e| format!("Failed to serialize message: {e}"))
}

/// Parses a client message by reading the envelope version and decoding the payload into the
/// current `ClientToServer` type.
///
/// This function does **only parsing** and returns:
/// - the `Version` found in the message
/// - the parsed message as `peer_practice_messages::current::messages::ClientToServer`
fn parse_received_message(text: &Utf8Bytes) -> eyre::Result<(Version, ClientToServer)> {
    fn parse_and_upgrade(text: &Utf8Bytes, version: Version) -> eyre::Result<ClientToServer> {
        match version {
            Version::V2026_02_07 => Ok(serde_json::from_str::<Envelope<ClientToServer>>(text)
                .wrap_err("Failed to parse Envelope")?
                .data),

            Version::V2026_01_11 => {
                let v01 = serde_json::from_str::<
                    Envelope<peer_practice_messages::v2026_01_11::messages::ClientToServer>,
                >(text)
                .wrap_err("Failed to parse Envelope")?
                .data;

                Ok(v01.into())
            }

            Version::V2025_10_14 => {
                let v10 = serde_json::from_str::<
                    Envelope<peer_practice_messages::v2025_10_14::messages::ClientToServer>,
                >(text)
                .wrap_err("Failed to parse Envelope")?
                .data;

                let v01: peer_practice_messages::v2026_01_11::messages::ClientToServer = v10.into();
                Ok(v01.into())
            }
        }
    }

    let client_version = serde_json::from_str::<EnvelopeHeader>(text)
        .wrap_err("Parsing message header failed")?
        .version;

    let data = parse_and_upgrade(text, client_version)?;

    Ok((client_version, data))
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    mut jar: CookieJar,
) -> Result<(CookieJar, Response), Response> {
    let (token, access_token) = retrieve_and_validate_access_token(&state, &mut jar).await?;
    let user_id = token.user_id;
    let session_id = match token.client_id {
        None => {
            let (tx, rx) = oneshot::channel();
            _ = state
                .active_sessions
                .send(ActiveSessionsMsg::CreateClient(user_id, tx))
                .await;
            match rx.await {
                Ok(client_id) => Ok(client_id),
                Err(_) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create session",
                )
                    .into_response()),
            }
        }
        Some(client_id) => Ok(client_id),
    }?;

    jar = create_access_cookie(&state, jar, user_id, Some(session_id))
        .map_err(|a| a.into_response())?;
    Ok((
        jar,
        ws.on_upgrade(move |socket| {
            handle_socket(socket, user_id, session_id, state, access_token)
        }),
    ))
}

async fn retrieve_and_validate_access_token(
    state: &AppState,
    jar: &mut CookieJar,
) -> Result<(Claims, String), Response> {
    let access_token = match jar.get("access_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Err((StatusCode::UNAUTHORIZED, "No access token").into_response());
        }
    };

    let (tx, rx) = oneshot::channel();
    _ = state
        .active_sessions
        .send(ActiveSessionsMsg::ValidateJwt(access_token, tx))
        .await;

    // timing attack possibly?
    let access_token = match rx.await {
        Ok(Some(token)) => token,
        _ => {
            return Err((StatusCode::UNAUTHORIZED, "Invalid token").into_response());
        }
    };

    let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_ref());
    let token = match decode::<Claims>(&access_token, &decoding_key, &Validation::default()) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => {
            error!("Failed to decode JWT: {}", e);
            Err((StatusCode::UNAUTHORIZED, "Invalid token").into_response())
        }
    }?;
    Ok((token, access_token))
}

async fn handle_socket(
    mut socket: WebSocket,
    user_id: UserId,
    session_id: SessionId,
    state: AppState,
    access_token: String,
) {
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
    let mut client_version = None;

    let mut invalidated = false;

    loop {
        tokio::select! {
            server_message = hub_rx.recv() => {
              if send_websocket_message(server_message, &mut socket, &client_version).await.is_none() {
                    break;
                };

                let (tx, rx) = oneshot::channel();
                _ = state.active_sessions
                    .send(ActiveSessionsMsg::GetSessionState(user_id, session_id, tx))
                    .await;

                if let Ok(SessionState::LoggedOut) = rx.await {
                    break;
                }
            }
            client_message = socket.recv() => {
                match receive_websocket_message(client_message, connection_id, user_id, session_id, &client_version, &state).await {
                    Some(version) => client_version = Some(version),
                    None => break,
                }

                let (tx, rx) = oneshot::channel();
                _ = state.active_sessions.send(ActiveSessionsMsg::GetSessionState(user_id, session_id, tx)).await;

                if let Ok(SessionState::LoggedOut) = rx.await {
                    break;
                }

                if !invalidated {
                    _ = state
                        .active_sessions
                        .send(ActiveSessionsMsg::InvalidateJwt(access_token.clone()))
                        .await;
                    invalidated = true;
                }
            }
        }
    }
}

async fn send_websocket_message(
    server_message: Option<ServerToClient>,
    socket: &mut WebSocket,
    client_version: &Option<Version>,
) -> Option<()> {
    match server_message {
        Some(server_msg) => {
            if let Some(client_version) = *client_version {
                let res = send_message(socket, server_msg, client_version).await;
                match res {
                    Ok(()) => Some(()),
                    Err(e) => {
                        error!("Failed to send message to client: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        }
        None => {
            None
        }
    }
}

async fn receive_websocket_message(
    ws_msg: Option<Result<Message, Error>>,
    connection_id: ConnectionId,
    user_id: UserId,
    session_id: SessionId,
    expected_client_version: &Option<Version>,
    state: &AppState,
) -> Option<Version> {
    match ws_msg {
        Some(Ok(Message::Text(text))) => {
            match consume_telegram(
                expected_client_version,
                &text,
                connection_id,
                user_id,
                session_id,
                state,
            )
            .await
            {
                Ok(version) => Some(version),
                Err(rep) => {
                    error!("Failed to consume message: {}", rep);
                    None
                }
            }
        }
        Some(Ok(Message::Close(_))) => {
            info!(
                "WebSocket connection closed for {:?} at {:?}.",
                user_id, connection_id
            );
            None
        }
        Some(Ok(_)) => {
            info!("Received unexpected message type, ignoring.");
            *expected_client_version
        }
        Some(Err(e)) => {
            error!("WebSocket error for {:?}: {}", user_id, e);
            None
        }
        None => None,
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
async fn consume_telegram(
    expected_client_version: &Option<Version>,
    text: &Utf8Bytes,
    connection_id: ConnectionId,
    user_id: UserId,
    session_id: SessionId,
    state: &AppState,
) -> eyre::Result<Version> {
    info!("Received message from {:?}: {}", user_id, text);

    if let Some(expected) = expected_client_version {
        let actual = serde_json::from_str::<EnvelopeHeader>(text)
            .wrap_err("Parsing message header failed")?
            .version;

        if actual != *expected {
            eyre::bail!("Expected version {:?}, got {:?}", expected, actual);
        }
    }

    let (client_version, data) = parse_received_message(text)?;

    handle_websocket_message(connection_id, state, user_id, session_id, data)
        .await
        .wrap_err("Failed to handle websocket message")?;

    Ok(client_version)
}

#[cfg(feature = "fuzzing")]
pub fn parse_received_message_for_fuzz(text: &str) -> eyre::Result<(Version, ClientToServer)> {
    parse_received_message(&Utf8Bytes::from(text))
}

#[cfg(test)]
mod tests;
