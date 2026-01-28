use crate::chat::ChatMsg;
use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::server_to_client::PostAction;
use peer_practice_messages::current::post::{Post, PostId};
use peer_practice_messages::current::user::UserId;
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum PostsMsg {
    New(Post, oneshot::Sender<PostId>),
    Upsert(PostId, Post),
    UserJoins(PostId, UserId),
    UserLeaves(PostId, UserId),
    Remove(PostId),
    Get(PostId, oneshot::Sender<Option<Post>>),
    List(oneshot::Sender<Vec<(PostId, Post)>>),
    Ping(oneshot::Sender<()>),
}

#[cfg(test)]
mod test;

pub async fn handle_posts(
    storage: Sender<StorageMsg>,
    ws_hub: Sender<WsHubMsg>,
    chat: Sender<ChatMsg>,
    mut rx: Receiver<PostsMsg>,
) {
    let mut posts: HashMap<PostId, Post> = HashMap::new();

    setup(&storage, &mut posts).await;

    while let Some(msg) = rx.recv().await {
        match msg {
            PostsMsg::Upsert(id, post) => {
                posts.insert(id, post.clone());
                let _ = ws_hub
                    .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                        PostAction::Post(id, post),
                    )))
                    .await;
                let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
            }
            PostsMsg::Remove(id) => {
                posts.remove(&id);
                let _ = ws_hub
                    .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                        PostAction::RemovedPost(id),
                    )))
                    .await;
                let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                let _ = chat.send(ChatMsg::DeleteForPost(id)).await;
            }
            PostsMsg::Get(id, reply) => {
                let result = posts.get(&id).cloned();
                let _ = reply.send(result);
            }
            PostsMsg::List(reply) => {
                let list = posts.iter().map(|(id, post)| (*id, post.clone())).collect();
                let _ = reply.send(list);
            }
            PostsMsg::New(post, sender) => {
                let id = PostId::new();
                posts.insert(id, post.clone());
                let _ = sender.send(id);
                let _ = ws_hub
                    .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                        PostAction::Post(id, post),
                    )))
                    .await;
                let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                let _ = chat.send(ChatMsg::CreateForPost(id)).await;
            }
            PostsMsg::UserJoins(post_id, user) => {
                if let Some(post) = posts.get_mut(&post_id) {
                    let joined = post.partaking_users.insert(user);
                    let _ = ws_hub
                        .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                            PostAction::Post(post_id, post.clone()),
                        )))
                        .await;
                    let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                    if joined {
                        let _ = chat
                            .send(ChatMsg::StoreMsgForPost {
                                post_id,
                                sender: user,
                                kind: peer_practice_messages::current::chat::ChatMessageKind::Joined,
                            })
                            .await;
                    }
                }
            }
            PostsMsg::UserLeaves(post_id, user) => {
                if let Some(post) = posts.get_mut(&post_id) {
                    let left = post.partaking_users.remove(&user);
                    let _ = ws_hub
                        .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                            PostAction::Post(post_id, post.clone()),
                        )))
                        .await;
                    let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                    if left {
                        let _ = chat
                            .send(ChatMsg::StoreMsgForPost {
                                post_id,
                                sender: user,
                                kind: peer_practice_messages::current::chat::ChatMessageKind::Left,
                            })
                            .await;
                    }
                }
            }
            PostsMsg::Ping(respond_to) => {
                let _ = respond_to.send(());
            }
        }
    }
}

async fn setup(storage: &Sender<StorageMsg>, posts: &mut HashMap<PostId, Post>) {
    let (respond_to, recv) = oneshot::channel();
    let _ = storage.send(StorageMsg::RetrievePosts { respond_to }).await;

    if let Ok(snapshot) = recv.await {
        snapshot.into_iter().for_each(|(id, post)| {
            posts.insert(id, post);
        })
    }
}
