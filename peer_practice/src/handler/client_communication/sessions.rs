use crate::app_state::AppState;
use crate::handler::client_communication::Response;
use eyre::Context;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::client_to_server as cts;
use peer_practice_messages::current::messages::server_to_client as stc;
use peer_practice_messages::current::user::UserId;
use peer_practice_messages::v2026_02_07::sessions::SessionId;
use peer_practice_server_services::active_sessions;
use tokio::sync::oneshot;

pub(crate) async fn sessions_handler(
    action: cts::SessionAction,
    state: &AppState,
    user_id: UserId,
    session_id: SessionId,
) -> eyre::Result<Response> {
    let response = match action {
        cts::SessionAction::GetSessions => {
            let (tx, rx) = oneshot::channel();
            state
                .active_sessions
                .send(active_sessions::ActiveSessionsMsg::GetSessions(user_id, tx))
                .await?;
            let sessions = rx
                .await
                .wrap_err("Failed to receive active sessions from server")?;
            Some(vec![ServerToClient::Session(stc::SessionAction::Sessions(
                sessions,
            ))])
        }
        cts::SessionAction::GetThisSession => Some(vec![ServerToClient::Session(
            stc::SessionAction::CurrentSession(session_id),
        )]),
        cts::SessionAction::UpdateSession(session) => {
            state
                .active_sessions
                .send(active_sessions::ActiveSessionsMsg::UpdateSession(
                    user_id, session,
                ))
                .await?;
            None
        }
        cts::SessionAction::LogOutSession(id) => {
            state
                .active_sessions
                .send(active_sessions::ActiveSessionsMsg::LogOut(user_id, id))
                .await?;
            None
        }
        cts::SessionAction::LogOutAllSessions => {
            state
                .active_sessions
                .send(active_sessions::ActiveSessionsMsg::LogOutAll(user_id))
                .await?;
            None
        }
    };
    Ok(response)
}
