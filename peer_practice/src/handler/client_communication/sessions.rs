use crate::app_state::AppState;
use crate::handler::client_communication::Response;
use peer_practice_messages::current::messages::client_to_server::SessionAction;
use peer_practice_messages::current::user::UserId;

pub(crate) async fn sessions_handler(
    action: SessionAction,
    state: &AppState,
    user_id: UserId,
) -> eyre::Result<Response> {
    todo!()
}
