use crate::app_state::AppState;
use peer_practice_messages::current::messages::client_to_server::ChatAction;
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::ws_hub::ConnectionId;

pub(crate) async fn chat_handler(
    action: ChatAction,
    state: &AppState,
    user: UserId,
    connection: ConnectionId,
) -> eyre::Result<()> {
    // match action {
    //     ChatAction::GetChatFor(post) => {
    //         let chat = state.chat.get_chat_for(post)?;
    //         state.ws_hub.send_message(connection, ChatMsg::Chat(chat))?;
    //     }
    //     ChatAction::GetChat(_) => {}
    //     ChatAction::SendMessage(_) => {}
    // }
    Ok(())
}
