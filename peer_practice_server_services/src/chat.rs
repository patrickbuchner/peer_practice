use crate::chat::message::Message;
use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::chat::ChatId;
use peer_practice_messages::current::post::PostId;
use progress::Progress;
use tokio::sync::{mpsc, oneshot};

pub mod message;
pub mod progress;

#[derive(Debug)]
pub enum ChatMsg {
    GetChatForPost(PostId, oneshot::Sender<Result<Progress, ()>>),
    GetChat(ChatId, oneshot::Sender<Result<Progress, ()>>),
    StoreMsg(Message),
}

pub async fn handle_chats(
    storage: mpsc::Sender<StorageMsg>,
    ws_hub: mpsc::Sender<WsHubMsg>,
    mut rx: mpsc::Receiver<ChatMsg>,
) {
}
