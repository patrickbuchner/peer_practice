use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::post::{Post, PostId};
use peer_practice_messages::current::user::UserId;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum PostsMsg {
    New(Post, oneshot::Sender<PostId>),
    Upsert(PostId, Post),
    UserJoins(PostId, UserId),
    UserLeaves(PostId, UserId),
    Remove(PostId),
    Get(PostId, oneshot::Sender<Option<Post>>),
    List(oneshot::Sender<Vec<(PostId, Post)>>),
}

pub fn spawn_posts_actor(
    storage: Sender<StorageMsg>,
    ws_hub: Sender<WsHubMsg>,
) -> Sender<PostsMsg> {
    let (tx, mut rx) = mpsc::channel::<PostsMsg>(100);

    tokio::spawn(async move {
        let mut posts: HashMap<PostId, Post> = HashMap::new();

        setup(&storage, &mut posts).await;

        while let Some(msg) = rx.recv().await {
            match msg {
                PostsMsg::Upsert(id, post) => {
                    posts.insert(id, post.clone());
                    let _ = ws_hub
                        .send(WsHubMsg::BroadcastAll(ServerToClient::Post(id, post)))
                        .await;
                    let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                }
                PostsMsg::Remove(id) => {
                    posts.remove(&id);
                    let _ = ws_hub
                        .send(WsHubMsg::BroadcastAll(ServerToClient::RemovedPost(id)))
                        .await;
                    let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
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
                        .send(WsHubMsg::BroadcastAll(ServerToClient::Post(id, post)))
                        .await;
                    let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                }
                PostsMsg::UserJoins(post_id, user) => {
                    if let Some(post) = posts.get_mut(&post_id) {
                        post.partaking_users.insert(user);
                        let _ = ws_hub
                            .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                                post_id,
                                post.clone(),
                            )))
                            .await;
                        let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                    }
                }
                PostsMsg::UserLeaves(post_id, user) => {
                    if let Some(post) = posts.get_mut(&post_id) {
                        post.partaking_users.remove(&user);
                        let _ = ws_hub
                            .send(WsHubMsg::BroadcastAll(ServerToClient::Post(
                                post_id,
                                post.clone(),
                            )))
                            .await;
                        let _ = storage.send(StorageMsg::SavePosts(posts.clone())).await;
                    }
                }
            }
        }
    });

    tx
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
