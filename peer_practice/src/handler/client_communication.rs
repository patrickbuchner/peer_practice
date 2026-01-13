use crate::app_state::AppState;
use eyre::Context;
use peer_practice_messages::current::messages::server_to_client::UserAction;
use peer_practice_messages::current::messages::{ClientToServer, ServerToClient};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use tracing::info;

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
        ClientToServer::Chat(_) => todo!(),
        ClientToServer::MessageNotYetKnown => {}
    }
    Ok(())
}
