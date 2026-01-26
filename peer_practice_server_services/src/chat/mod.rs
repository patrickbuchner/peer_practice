use crate::chat::message::Message;
use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::chat::ChatId;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::server_to_client::ChatAction;
use peer_practice_messages::current::post::PostId;
use progress::Progress;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

pub mod message;
pub mod progress;
#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum ChatMsg {
    GetChatForPost(PostId, oneshot::Sender<Result<Progress, ()>>),
    GetChat(ChatId, oneshot::Sender<Result<Progress, ()>>),
    StoreMsg(Message),
    CreateForPost(PostId),
    DeleteForPost(PostId),
    Delete(ChatId),
}

pub async fn handle_chats(
    storage: mpsc::Sender<StorageMsg>,
    ws_hub: mpsc::Sender<WsHubMsg>,
    mut rx: mpsc::Receiver<ChatMsg>,
) {
    let mut chats: HashMap<ChatId, Progress> = HashMap::new();
    let mut post_to_chat: HashMap<PostId, ChatId> = HashMap::new();

    setup(&storage, &mut chats, &mut post_to_chat).await;

    while let Some(msg) = rx.recv().await {
        match msg {
            ChatMsg::GetChatForPost(post_id, respond_to) => {
                let result = if post_to_chat.contains_key(&post_id) {
                    let chat_id = post_to_chat[&post_id];
                    chats.get(&chat_id).cloned().ok_or(())
                } else {
                    Err(())
                };
                let _ = respond_to.send(result);
            }
            ChatMsg::GetChat(chat_id, respond_to) => {
                let result = chats.get(&chat_id).cloned().ok_or(());
                let _ = respond_to.send(result);
            }
            ChatMsg::CreateForPost(post_id) => {
                if let std::collections::hash_map::Entry::Vacant(e) = post_to_chat.entry(post_id) {
                    let chat_id = ChatId::new();
                    e.insert(chat_id);
                    chats.insert(
                        chat_id,
                        Progress {
                            chat_id,
                            post_id,
                            content: Vec::new(),
                        },
                    );
                    let _ = storage.send(StorageMsg::SaveChats(chats.clone())).await;
                }
            }
            ChatMsg::DeleteForPost(post_id) => {
                if let Some(chat_id) = post_to_chat.remove(&post_id)
                    && chats.remove(&chat_id).is_some()
                {
                    let _ = storage.send(StorageMsg::SaveChats(chats.clone())).await;
                }
            }
            ChatMsg::Delete(chat_id) => {
                if let Some(progress) = chats.remove(&chat_id) {
                    post_to_chat.remove(&progress.post_id);
                    let _ = storage.send(StorageMsg::SaveChats(chats.clone())).await;
                }
            }
            ChatMsg::StoreMsg(message) => {
                if let Some(progress) = chats.get_mut(&message.chat_id) {
                    let outgoing = ServerToClient::Chat(ChatAction::MessageSent((&message).into()));
                    progress.content.push(message);
                    let _ = ws_hub.send(WsHubMsg::BroadcastAll(outgoing)).await;
                    let _ = storage.send(StorageMsg::SaveChats(chats.clone())).await;
                }
            }
        }
    }
}

async fn setup(
    storage: &mpsc::Sender<StorageMsg>,
    chats: &mut HashMap<ChatId, Progress>,
    post_to_chat: &mut HashMap<PostId, ChatId>,
) {
    let (respond_to, recv) = oneshot::channel();
    let _ = storage.send(StorageMsg::RetrieveChats { respond_to }).await;
    if let Ok(snapshot) = recv.await {
        for (chat_id, progress) in snapshot {
            post_to_chat.insert(progress.post_id, chat_id);
            chats.insert(chat_id, progress);
        }
    }
}
