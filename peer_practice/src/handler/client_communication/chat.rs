use crate::app_state::AppState;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::client_to_server::ChatAction;
use peer_practice_messages::current::messages::server_to_client::ChatAction::{
    Chat, ChatDoesNotExist, ChatDoesNotExistForPost,
};
use peer_practice_messages::current::user::UserId;
use peer_practice_server_services::chat::{ChatMsg, ensure_chat_for_post};
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
            let chat_id = ensure_chat_for_post(&state.chat, post).await;
            let msg = if let Some(chat_id) = chat_id {
                let (sender, receiver) = oneshot::channel();
                state.chat.send(ChatMsg::GetChat(chat_id, sender)).await?;
                match receiver.await {
                    Ok(Ok(m)) => ServerToClient::Chat(Chat(
                        m.chat_id,
                        m.post_id,
                        m.content.iter().map(|m| m.into()).collect(),
                    )),
                    _ => ServerToClient::Chat(ChatDoesNotExistForPost(post)),
                }
            } else {
                ServerToClient::Chat(ChatDoesNotExistForPost(post))
            };

            send_chat(state, user_id, con_id, msg).await?;
        }
        ChatAction::GetChat(id) => {
            let (sender, receiver) = oneshot::channel();
            state.chat.send(ChatMsg::GetChat(id, sender)).await?;
            let msg = match receiver.await {
                Ok(Ok(m)) => ServerToClient::Chat(Chat(
                    m.chat_id,
                    m.post_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::test_utils::{recv_msg, test_state};
    use peer_practice_messages::current::messages::ServerToClient;
    use peer_practice_messages::current::post::PostId;
    use peer_practice_messages::test_helpers_impl::fixed_timestamp;
    use peer_practice_server_services::chat::message::Message;
    use peer_practice_server_services::chat::progress::Progress;
    use peer_practice_server_services::ws_hub::ConnectionId;

    #[tokio::test]
    async fn get_chat_for_missing_sends_not_found() {
        let (state, mut rx) = test_state();
        let user_id = UserId::new();
        let con_id = ConnectionId::new();
        let post_id = PostId::new();

        let state = state.clone();
        let handler = tokio::spawn(async move {
            chat_handler(ChatAction::GetChatFor(post_id), &state, user_id, con_id).await
        });

        match recv_msg(&mut rx.chat).await {
            ChatMsg::GetChatForPost(got_post, respond_to) => {
                assert_eq!(post_id, got_post);
                let _ = respond_to.send(Err(()));
            }
            _ => panic!("expected ChatMsg::GetChatForPost"),
        }

        if let ChatMsg::CreateForPost(got_post) = recv_msg(&mut rx.chat).await {
            assert_eq!(post_id, got_post);
        } else {
            panic!("expected CreateForPost");
        }

        if let ChatMsg::GetChatForPost(got_post, respond_to) = recv_msg(&mut rx.chat).await {
            assert_eq!(post_id, got_post);
            let _ = respond_to.send(Err(()));
        } else {
            panic!("expected GetChatForPost after CreateForPost");
        }

        handler.await.expect("handler task ok").expect("handler ok");

        match recv_msg(&mut rx.ws_hub).await {
            WsHubMsg::Send { msg, .. } => match msg {
                ServerToClient::Chat(ChatDoesNotExistForPost(got_post)) => {
                    assert_eq!(post_id, got_post);
                }
                _ => panic!("expected ChatDoesNotExistForPost"),
            },
            _ => panic!("expected WsHubMsg::Send"),
        }
    }

    #[tokio::test]
    async fn get_chat_sends_chat_contents() {
        let (state, mut rx) = test_state();
        let user_id = UserId::new();
        let con_id = ConnectionId::new();
        let post_id = PostId::new();
        let chat_id = peer_practice_messages::current::chat::ChatId::new();
        let progress = Progress {
            chat_id,
            post_id,
            content: vec![Message {
                sender: user_id,
                kind: peer_practice_messages::current::chat::ChatMessageKind::Text(
                    "hello".to_string(),
                ),
                chat_id,
                timestamp: fixed_timestamp(),
            }],
        };

        let state = state.clone();
        let handler = tokio::spawn(async move {
            chat_handler(ChatAction::GetChat(chat_id), &state, user_id, con_id).await
        });

        match recv_msg(&mut rx.chat).await {
            ChatMsg::GetChat(got_id, respond_to) => {
                assert_eq!(chat_id, got_id);
                let _ = respond_to.send(Ok(progress.clone()));
            }
            _ => panic!("expected ChatMsg::GetChat"),
        }

        handler.await.expect("handler task ok").expect("handler ok");

        match recv_msg(&mut rx.ws_hub).await {
            WsHubMsg::Send { msg, .. } => match msg {
                ServerToClient::Chat(Chat(got_id, got_post_id, messages)) => {
                    assert_eq!(chat_id, got_id);
                    assert_eq!(post_id, got_post_id);
                    assert_eq!(1, messages.len());
                    assert!(matches!(
                        messages[0].kind,
                        peer_practice_messages::current::chat::ChatMessageKind::Text(ref text)
                            if text == "hello"
                    ));
                }
                _ => panic!("expected Chat message"),
            },
            _ => panic!("expected WsHubMsg::Send"),
        }
    }
}
