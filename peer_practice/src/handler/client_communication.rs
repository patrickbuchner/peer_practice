use crate::app_state::AppState;
use eyre::Context;
use peer_practice_messages::current::messages::server_to_client::UserAction;
use peer_practice_messages::current::messages::{ClientToServer, ServerToClient};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use tracing::info;

mod chat;
mod posts;
mod users;

pub async fn handle_websocket_message(
    con_id: ConnectionId,
    state: &AppState,
    user_id: UserId,
    msg: ClientToServer,
) -> eyre::Result<()> {
    info!("Received message from client: {:?} {:?}", msg, con_id);
    match msg {
        ClientToServer::Hello => {
            state
                .ws_hub
                .send(WsHubMsg::Send {
                    user_id,
                    con_id,
                    msg: ServerToClient::User(UserAction::YouAre(user_id)),
                })
                .await
                .wrap_err("Failed to send hello message to client")?;
        }
        ClientToServer::User(action) => users::user_handler(action, state, user_id, con_id)
            .await
            .wrap_err("Failed to handle user action")?,
        ClientToServer::Post(action) => posts::post_handler(action, state, user_id, con_id)
            .await
            .wrap_err("Failed to handle post action")?,
        ClientToServer::Chat(action) => chat::chat_handler(action, state, user_id, con_id)
            .await
            .wrap_err("Failed to handle chat action")?,
        ClientToServer::MessageNotYetKnown => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::test_utils::{recv_msg, test_state};
    use peer_practice_messages::current::messages::server_to_client::UserAction;
    use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};

    #[tokio::test]
    async fn hello_sends_you_are() {
        let (state, mut rx) = test_state();
        let user_id = UserId::new();
        let con_id = ConnectionId::new();

        handle_websocket_message(con_id, &state, user_id, ClientToServer::Hello)
            .await
            .expect("handler ok");

        match recv_msg(&mut rx.ws_hub).await {
            WsHubMsg::Send {
                user_id: got_user,
                con_id: got_con,
                msg,
            } => {
                assert_eq!(user_id, got_user);
                assert_eq!(con_id, got_con);
                match msg {
                    ServerToClient::User(UserAction::YouAre(got_id)) => assert_eq!(user_id, got_id),
                    _ => panic!("expected YouAre"),
                }
            }
            _ => panic!("expected WsHubMsg::Send"),
        }
    }
}
