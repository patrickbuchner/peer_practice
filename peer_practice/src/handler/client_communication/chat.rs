use crate::app_state::AppState;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::client_to_server::ChatAction;
use peer_practice_messages::current::messages::server_to_client::ChatAction::{
    Chat, ChatDoesNotExist, ChatDoesNotExistForPost,
};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::chat::ChatMsg;
use peer_practice_server_services::ws_hub::{ConnectionId, WsHubMsg};
use tokio::sync::oneshot;

pub(crate) async fn chat_handler(
    action: ChatAction,
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
) -> eyre::Result<()> {
    match action {
        ChatAction::GetChatFor(post) => {
            let (sender, receiver) = oneshot::channel();
            state
                .chat
                .send(ChatMsg::GetChatForPost(post, sender))
                .await?;
            let msg = match receiver.await {
                Ok(Ok(m)) => ServerToClient::Chat(Chat(
                    m.chat_id,
                    m.content.iter().map(|m| m.into()).collect(),
                )),
                _ => ServerToClient::Chat(ChatDoesNotExistForPost(post)),
            };

            send_chat(state, user_id, con_id, msg).await?;
        }
        ChatAction::GetChat(id) => {
            let (sender, receiver) = oneshot::channel();
            state.chat.send(ChatMsg::GetChat(id, sender)).await?;
            let msg = match receiver.await {
                Ok(Ok(m)) => ServerToClient::Chat(Chat(
                    m.chat_id,
                    m.content.iter().map(|m| m.into()).collect(),
                )),
                _ => ServerToClient::Chat(ChatDoesNotExist(id)),
            };
            send_chat(state, user_id, con_id, msg).await?;
        }
        ChatAction::SendMessage(msg) => {
            state.chat.send(ChatMsg::StoreMsg(msg.into())).await?;
        }
    }
    Ok(())
}

async fn send_chat(
    state: &AppState,
    user_id: UserId,
    con_id: ConnectionId,
    msg: ServerToClient,
) -> eyre::Result<()> {
    state
        .ws_hub
        .send(WsHubMsg::Send {
            user_id,
            con_id,
            msg,
        })
        .await?;
    Ok(())
}
