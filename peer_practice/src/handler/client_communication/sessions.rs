use crate::app_state::AppState;
use peer_practice_messages::current::messages::client_to_server::SessionAction;
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::ws_hub::ConnectionId;
use crate::handler::client_communication::Response;

pub(crate) async fn sessions_handler(
    action: SessionAction,
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
) -> eyre::Result<Response> {
    todo!()
}
