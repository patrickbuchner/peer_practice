use crate::app_state::AppState;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use peer_practice_shared::messages::server_to_client::UserAction;
use peer_practice_shared::messages::{ClientToServer, ServerToClient};
use peer_practice_shared::user::UserId;

mod posts;
mod users;

pub async fn handle_websocket_message(
    con_id: ConnectionId,
    state: &AppState,
    user_id: UserId,
    msg: ClientToServer,
) {
    match msg {
        ClientToServer::Hello => {
            _ = state
                .ws_hub
                .send(WsHubMsg::Send {
                    user_id,
                    con_id,
                    msg: ServerToClient::User(UserAction::YouAre(user_id)),
                })
                .await;
        }
        ClientToServer::User(action) => users::user_handler(action, state, user_id, con_id).await,
        ClientToServer::Post(action) => posts::post_handler(action, state, user_id, con_id).await,
        ClientToServer::Chat(_) => todo!(),
    }
}
