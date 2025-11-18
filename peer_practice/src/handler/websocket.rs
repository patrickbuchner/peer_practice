use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, Validation, decode};
use tokio::sync::oneshot;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::handler::claims::Claims;
use crate::handler::client_communication::handle_websocket_message;
use peer_practice_server_services::ws_hub::WsHubMsg;
use peer_practice_shared::messages::{ClientToServer, ServerToClient};
use peer_practice_shared::user::UserId;

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

    let (_connection_handle, mut hub_rx) = match rx.await {
        Ok(result) => result,
        Err(e) => {
            error!(
                "Failed to register WS connection in hub for {:?}: {}",
                user_id, e
            );
            return;
        }
    };
    if socket
        .send(Message::Text(
            serde_json::to_string(&ServerToClient::YouAre(user_id))
                .unwrap()
                .into(),
        ))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            maybe_msg = hub_rx.recv() => {
                match maybe_msg {
                    Some(server_msg) => {
                        let Ok(text) = serde_json::to_string(&server_msg) else {
                            error!("Failed to serialize ServerToClient message for {:?}", user_id);
                            continue;
                        };
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            // Client disconnected
                            break;
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
                        info!("Received message from {:?}: {}", user_id, text);
                        match serde_json::from_str::<ClientToServer>(&text) {
                            Ok(msg) => {
                                handle_websocket_message(&mut socket, &state, user_id, msg).await;
                            }
                            Err(e) => {
                                error!("Failed to parse ClientToServer from {:?}: {}", user_id, e);
                            }
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
