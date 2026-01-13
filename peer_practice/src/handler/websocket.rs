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
    let text = match version {
        Version::V2026_01_11 => serde_json::to_string(&Envelope {
            version: Version::V2026_01_11,
            data: msg,
        }),
        Version::V2025_10_14 => serde_json::to_string(&Envelope::<
            peer_practice_messages::v2025_10_14::messages::ServerToClient,
        > {
            version: Version::V2025_10_14,
            data: msg.into(),
        }),
    };

    let text = text.map_err(|e| format!("Failed to serialize message: {}", e))?;

    info!("Sending message to client: {}", text);

    if socket.send(Message::Text(text.into())).await.is_err() {
        Err("Socket has been disconnected".into())
    } else {
        Ok(())
    }
}

async fn consume_message(
    connection_id: ConnectionId,
    state: &AppState,
    user_id: UserId,
    text: &Utf8Bytes,
    version: Version,
) -> eyre::Result<()> {
    info!("Received message from {:?}: {}", user_id, text);
    let data = match version {
        Version::V2026_01_11 => Some(
            serde_json::from_str::<Envelope<ClientToServer>>(text)
                .wrap_err("Failed to parse Envelope")?
                .data,
        ),
        Version::V2025_10_14 => None,
    };

    if let Some(data) = data {
        handle_websocket_message(connection_id, state, user_id, data)
            .await
            .wrap_err("Failed to handle websocket message")?;
    }
    Ok(())
}
