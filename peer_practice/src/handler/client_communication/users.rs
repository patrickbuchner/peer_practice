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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::test_utils::test_state;
    use peer_practice_messages::current::email::Email;
    use peer_practice_messages::current::messages::ServerToClient;
    use peer_practice_messages::current::messages::server_to_client::UserAction as ServerUserAction;
    use peer_practice_messages::current::user::User;
    use peer_practice_server_services::ws_hub::ConnectionId;
    use tokio::sync::mpsc::error::TryRecvError;
    use tokio::sync::oneshot;

    async fn recv_msg<T>(rx: &mut tokio::sync::mpsc::Receiver<T>) -> T {
        match rx.recv().await {
            Some(msg) => msg,
            None => panic!("channel closed"),
        }
    }

    async fn sync_users(state: &AppState, rx: &mut tokio::sync::mpsc::Receiver<UsersMsg>) {
        let (respond_to, recv) = oneshot::channel();
        state
            .users
            .send(UsersMsg::Ping { respond_to })
            .await
            .expect("send ping");

        match recv_msg(rx).await {
            UsersMsg::Ping { respond_to } => {
                let _ = respond_to.send(());
            }
            _ => panic!("expected UsersMsg::Ping"),
        }

        recv.await.expect("ping ack");
    }

    fn assert_empty<T>(rx: &mut tokio::sync::mpsc::Receiver<T>) {
        match rx.try_recv() {
            Ok(_) => panic!("expected no message"),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("channel closed"),
        }
    }

    fn sample_user(id: UserId) -> User {
        User {
            id,
            email: Email::new("user@example.com").unwrap(),
            display_name: Some("Tester".to_string()),
        }
    }

    #[tokio::test]
    async fn update_user_ignores_mismatched_user_id() {
        let (state, mut rx) = test_state();
        let user_id = UserId::new();
        let other_id = UserId::new();
        let display = peer_practice_messages::current::user::display_user::UserDisplay {
            id: other_id,
            display_name: Some("New".to_string()),
        };

        user_handler(
            UserAction::Update(display),
            &state,
            user_id,
            ConnectionId::new(),
        )
        .await
        .expect("handler ok");

        sync_users(&state, &mut rx.users).await;
        assert_empty(&mut rx.users);
    }

    #[tokio::test]
    async fn get_user_sends_user_when_found() {
        let (state, mut rx) = test_state();
        let user_id = UserId::new();
        let con_id = ConnectionId::new();
        let user = sample_user(user_id);

        let state = state.clone();
        let handler = tokio::spawn(async move {
            user_handler(UserAction::Get(user_id), &state, user_id, con_id).await
        });

        match recv_msg(&mut rx.users).await {
            UsersMsg::GetById { id, respond_to } => {
                assert_eq!(user_id, id);
                let _ = respond_to.send(Some(user.clone()));
            }
            _ => panic!("expected UsersMsg::GetById"),
        }

        handler.await.expect("handler task ok").expect("handler ok");

        match recv_msg(&mut rx.ws_hub).await {
            WsHubMsg::Send {
                user_id: got_user,
                con_id: got_con,
                msg,
            } => {
                assert_eq!(user_id, got_user);
                assert_eq!(con_id, got_con);
                match msg {
                    ServerToClient::User(ServerUserAction::User(got_id, _)) => {
                        assert_eq!(user_id, got_id);
                    }
                    _ => panic!("expected User message"),
                }
            }
            _ => panic!("expected WsHubMsg::Send"),
        }
    }
}
