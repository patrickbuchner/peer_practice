use axum::Error;
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
    let mut client_version = Some(Version::default());

    loop {
        tokio::select! {
            server_message = hub_rx.recv() => {
              if send_websocket_message(server_message, &mut socket, &client_version).await.is_none() {
                    break;
                };
            }
            client_message = socket.recv() => {
                match receive_websocket_message(client_message, connection_id, user_id, &client_version, &state).await {
                    Some(version) => client_version = Some(version),
                    None => break,
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
            // Hub closed our channel; end the connection
            None
        }
    }
}

async fn receive_websocket_message(
    ws_msg: Option<Result<Message, Error>>,
    connection_id: ConnectionId,
    user_id: UserId,
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

async fn consume_telegram(
    expected_client_version: &Option<Version>,
    text: &Utf8Bytes,
    connection_id: ConnectionId,
    user_id: UserId,
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

    handle_websocket_message(connection_id, state, user_id, data)
        .await
        .wrap_err("Failed to handle websocket message")?;

    Ok(client_version)
}

/// Parses a client message by reading the envelope version and decoding the payload into the
/// current `ClientToServer` type.
///
/// This function does **only parsing** and returns:
/// - the `Version` found in the message
/// - the parsed message as `peer_practice_messages::current::messages::ClientToServer`
fn parse_received_message(text: &Utf8Bytes) -> eyre::Result<(Version, ClientToServer)> {
    let client_version = serde_json::from_str::<EnvelopeHeader>(text)
        .wrap_err("Parsing message header failed")?
        .version;

    let data = match client_version {
        Version::V2026_01_11 => {
            serde_json::from_str::<Envelope<ClientToServer>>(text)
                .wrap_err("Failed to parse Envelope")?
                .data
        }
        Version::V2025_10_14 => {
            let legacy = serde_json::from_str::<
                Envelope<peer_practice_messages::v2025_10_14::messages::ClientToServer>,
            >(text)
            .wrap_err("Failed to parse Envelope")?;
            legacy.data.into()
        }
    };

    Ok((client_version, data))
}

#[cfg(feature = "fuzzing")]
pub fn parse_received_message_for_fuzz(text: &str) -> eyre::Result<(Version, ClientToServer)> {
    parse_received_message(&Utf8Bytes::from(text))
}

#[cfg(test)]
mod tests;
