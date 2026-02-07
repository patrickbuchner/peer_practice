use crate::app_state::AppState;
use crate::handler::client_communication::Response;
use eyre::Context;
use peer_practice_messages::current::messages::client_to_server::UserAction;
use peer_practice_messages::current::messages::{server_to_client, ServerToClient};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::users::UsersMsg;
use tokio::sync::oneshot;
use tracing::instrument;

#[instrument(skip(state), fields(user_id = ?user_id))]
pub async fn user_handler(
    user_action: UserAction,
    state: &AppState,
    user_id: UserId,
) -> eyre::Result<Response> {
    let msg = match user_action {
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
                Some(vec![ServerToClient::User(
                    server_to_client::UserAction::User(user.id, user.into()),
                )])
            } else {
                None
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
            None
        }
    };
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::test_utils::{assert_empty, recv_msg, test_state};
    use peer_practice_messages::current::email::Email;
    use peer_practice_messages::current::messages::server_to_client::UserAction as ServerUserAction;
    use peer_practice_messages::current::messages::ServerToClient;
    use peer_practice_messages::current::user::User;
    use tokio::sync::oneshot;

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

        user_handler(UserAction::Update(display), &state, user_id)
            .await
            .expect("handler ok");

        sync_users(&state, &mut rx.users).await;
        assert_empty(&mut rx.users);
    }

    #[tokio::test]
    async fn get_user_sends_user_when_found() {
        let (state, mut rx) = test_state();
        let user_id = UserId::new();
        let user = sample_user(user_id);

        let state = state.clone();
        let handler =
            tokio::spawn(
                async move { user_handler(UserAction::Get(user_id), &state, user_id).await },
            );

        match recv_msg(&mut rx.users).await {
            UsersMsg::GetById { id, respond_to } => {
                assert_eq!(user_id, id);
                let _ = respond_to.send(Some(user.clone()));
            }
            _ => panic!("expected UsersMsg::GetById"),
        }

        let response = handler.await.expect("handler task ok").expect("handler ok");

        assert_eq!(response, Some(vec![ServerToClient::User(ServerUserAction::User(user_id, user.into()))]));
    }
}
