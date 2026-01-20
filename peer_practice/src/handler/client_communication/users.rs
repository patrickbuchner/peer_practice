use crate::app_state::AppState;
use eyre::Context;
use peer_practice_messages::current::messages::client_to_server::UserAction;
use peer_practice_messages::current::messages::{ServerToClient, server_to_client};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::users::UsersMsg;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use tokio::sync::oneshot;
use tracing::instrument;

#[instrument(skip(state), fields(user_id = ?user_id, con_id = ?con_id))]
pub async fn user_handler(
    user_action: UserAction,
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
) -> eyre::Result<()> {
    match user_action {
        UserAction::Get(user) => {
            let (tx, rx) = oneshot::channel();
            state
                .users
                .send(UsersMsg::GetById {
                    id: user,
                    respond_to: tx,
                })
                .await
                .wrap_err("Failed to get user")?;

            if let Ok(user) = rx.await
                && let Some(user) = &user
            {
                state
                    .ws_hub
                    .send(WsHubMsg::Send {
                        user_id,
                        con_id,
                        msg: ServerToClient::User(server_to_client::UserAction::User(
                            user.id,
                            user.into(),
                        )),
                    })
                    .await
                    .wrap_err("Failed to send user to client")?;
            }
        }
        UserAction::Update(user_display) => {
            if user_display.id == user_id {
                let (tx, rx) = oneshot::channel();
                state
                    .users
                    .send(UsersMsg::GetById {
                        id: user_id,
                        respond_to: tx,
                    })
                    .await
                    .wrap_err("Failed to get user")?;

                if let Ok(user) = rx.await
                    && let Some(user) = &user
                {
                    let mut user = user.clone();
                    user.display_name = user_display.display_name;
                    state
                        .users
                        .send(UsersMsg::Update { id: user_id, user })
                        .await
                        .wrap_err("Failed to update user")?;
                }
            }
        }
    }
    Ok(())
}
