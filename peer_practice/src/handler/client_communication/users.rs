use crate::app_state::AppState;
use peer_practice_server_services::users::UsersMsg;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use peer_practice_shared::messages::client_to_server::UserAction;
use peer_practice_shared::messages::{ServerToClient, server_to_client};
use peer_practice_shared::user::UserId;
use tokio::sync::oneshot;
use tracing::info;

pub async fn user_handler(
    user_action: UserAction,
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
) {
    match user_action {
        UserAction::Get(user) => {
            info!(
                user_id = ?user_id,
                target_user_id = ?user,
                command = "GetUser",
                "received client command"
            );
            let (tx, rx) = oneshot::channel();
            _ = state
                .users
                .send(UsersMsg::GetById {
                    id: user,
                    respond_to: tx,
                })
                .await;
            if let Ok(user) = rx.await
                && let Some(user) = &user
            {
                _ = state.ws_hub.send(WsHubMsg::Send {
                    user_id,
                    con_id,
                    msg: ServerToClient::User(server_to_client::UserAction::User(
                        user.id,
                        user.into(),
                    )),
                });
            }
        }
        UserAction::Update(user_display) => {
            info!(
                user_id = ?user_id,
                target_user_id = ?user_display.id,
                display_name = ?user_display.display_name,
                command = "UpdateUser",
                "received client command"
            );
            if user_display.id == user_id {
                let (tx, rx) = oneshot::channel();
                _ = state
                    .users
                    .send(UsersMsg::GetById {
                        id: user_id,
                        respond_to: tx,
                    })
                    .await;
                if let Ok(user) = rx.await
                    && let Some(user) = &user
                {
                    let mut user = user.clone();
                    user.display_name = user_display.display_name;
                    _ = state
                        .users
                        .send(UsersMsg::Update { id: user_id, user })
                        .await;
                }
            }
        }
    }
}
