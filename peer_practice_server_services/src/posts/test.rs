use super::{handle_posts, PostsMsg};
use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::level::Level;
use peer_practice_messages::current::messages::server_to_client::PostAction;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::post::{Post, PostId, Topics};
use peer_practice_messages::current::user::UserId;
use std::collections::{HashMap, HashSet};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

async fn next<T>(rx: &mut mpsc::Receiver<T>) -> T {
    timeout(Duration::from_millis(300), rx.recv())
        .await
        .expect("timed out")
        .expect("channel closed")
}

async fn arrange_empty() -> (
    mpsc::Sender<PostsMsg>,
    mpsc::Receiver<StorageMsg>,
    mpsc::Receiver<WsHubMsg>,
    JoinHandle<()>,
) {
    let (storage_tx, mut storage_rx) = mpsc::channel::<StorageMsg>(16);
    let (ws_hub_tx, ws_hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let (posts_tx, posts_rx) = mpsc::channel::<PostsMsg>(16);

    let task = tokio::spawn(handle_posts(storage_tx, ws_hub_tx, posts_rx));

    if let StorageMsg::RetrievePosts { respond_to } = next(&mut storage_rx).await {
        let _ = respond_to.send(HashMap::new());
    } else {
        panic!("expected RetrievePosts");
    }

    (posts_tx, storage_rx, ws_hub_rx, task)
}

fn mk_post(owner: UserId) -> Post {
    Post {
        title: Topics::default(),
        content: "hello".to_string(),
        level: Level::Beginner1,
        owner,
        date: chrono::Utc::now(),
        partaking_users: HashSet::new(),
    }
}

async fn get(posts_tx: &mpsc::Sender<PostsMsg>, post_id: PostId) -> Option<Post> {
    let (respond_to, recv) = oneshot::channel();
    posts_tx.send(PostsMsg::Get(post_id, respond_to)).await.unwrap();
    recv.await.unwrap()
}

async fn list(posts_tx: &mpsc::Sender<PostsMsg>) -> Vec<(PostId, Post)> {
    let (respond_to, recv) = oneshot::channel();
    posts_tx.send(PostsMsg::List(respond_to)).await.unwrap();
    recv.await.unwrap()
}

#[tokio::test]
async fn get_missing_returns_none() {
    let (posts_tx, _storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let missing = PostId::new();
    let got = get(&posts_tx, missing).await;

    assert!(got.is_none());

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn new_broadcasts_and_persists_and_returns_id() {
    let (posts_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let owner = UserId::default();
    let post = mk_post(owner);

    let (id_tx, id_rx) = oneshot::channel();
    posts_tx.send(PostsMsg::New(post.clone(), id_tx)).await.unwrap();

    let id = timeout(Duration::from_millis(300), id_rx)
        .await
        .expect("timed out")
        .expect("oneshot closed");

    match next(&mut ws_hub_rx).await {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, got_post))) => {
            assert_eq!(id, got_id);
            assert_eq!(post.content, got_post.content);
        }
        other => panic!("expected Post broadcast, got {other:?}"),
    }

    match next(&mut storage_rx).await {
        StorageMsg::SavePosts(snapshot) => {
            assert!(snapshot.contains_key(&id), "saved snapshot should contain new post id");
        }
        other => panic!("expected SavePosts, got {other:?}"),
    }

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn upsert_then_get_returns_post() {
    let (posts_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let id = PostId::new();
    let owner = UserId::default();
    let post = mk_post(owner);

    posts_tx.send(PostsMsg::Upsert(id, post.clone())).await.unwrap();

    match next(&mut ws_hub_rx).await {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, _))) => {
            assert_eq!(id, got_id);
        }
        other => panic!("expected Post broadcast, got {other:?}"),
    }
    match next(&mut storage_rx).await {
        StorageMsg::SavePosts(snapshot) => assert!(snapshot.contains_key(&id)),
        other => panic!("expected SavePosts, got {other:?}"),
    }

    let got = get(&posts_tx, id).await.expect("post should exist");
    assert_eq!(post.content, got.content);

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn remove_broadcasts_and_persists() {
    let (posts_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let id = PostId::new();
    let post = mk_post(UserId::default());

    posts_tx.send(PostsMsg::Upsert(id, post)).await.unwrap();
    let _ = next(&mut ws_hub_rx).await;
    let _ = next(&mut storage_rx).await;

    posts_tx.send(PostsMsg::Remove(id)).await.unwrap();

    match next(&mut ws_hub_rx).await {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::RemovedPost(got_id))) => {
            assert_eq!(id, got_id);
        }
        other => panic!("expected RemovedPost broadcast, got {other:?}"),
    }

    match next(&mut storage_rx).await {
        StorageMsg::SavePosts(snapshot) => {
            assert!(
                !snapshot.contains_key(&id),
                "deleted post should not be present in saved snapshot"
            );
        }
        other => panic!("expected SavePosts, got {other:?}"),
    }

    assert!(get(&posts_tx, id).await.is_none());

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn join_then_leave_updates_partaking_users_and_persists() {
    let (posts_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let id = PostId::new();
    let owner = UserId::default();
    let mut post = mk_post(owner);
    post.partaking_users = HashSet::new();
    posts_tx.send(PostsMsg::Upsert(id, post)).await.unwrap();
    let _ = next(&mut ws_hub_rx).await;
    let _ = next(&mut storage_rx).await;

    let user = UserId::default();

    posts_tx.send(PostsMsg::UserJoins(id, user)).await.unwrap();

    match next(&mut ws_hub_rx).await {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, got_post))) => {
            assert_eq!(id, got_id);
            assert!(got_post.partaking_users.contains(&user));
        }
        other => panic!("expected Post broadcast after join, got {other:?}"),
    }
    match next(&mut storage_rx).await {
        StorageMsg::SavePosts(snapshot) => {
            let saved = snapshot.get(&id).unwrap();
            assert!(saved.partaking_users.contains(&user));
        }
        other => panic!("expected SavePosts after join, got {other:?}"),
    }

    posts_tx.send(PostsMsg::UserLeaves(id, user)).await.unwrap();

    match next(&mut ws_hub_rx).await {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, got_post))) => {
            assert_eq!(id, got_id);
            assert!(!got_post.partaking_users.contains(&user));
        }
        other => panic!("expected Post broadcast after leave, got {other:?}"),
    }
    match next(&mut storage_rx).await {
        StorageMsg::SavePosts(snapshot) => {
            let saved = snapshot.get(&id).unwrap();
            assert!(!saved.partaking_users.contains(&user));
        }
        other => panic!("expected SavePosts after leave, got {other:?}"),
    }

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn list_returns_all_posts() {
    let (posts_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let id1 = PostId::new();
    let id2 = PostId::new();

    posts_tx
        .send(PostsMsg::Upsert(id1, mk_post(UserId::default())))
        .await
        .unwrap();
    let _ = next(&mut ws_hub_rx).await;
    let _ = next(&mut storage_rx).await;

    posts_tx
        .send(PostsMsg::Upsert(id2, mk_post(UserId::default())))
        .await
        .unwrap();
    let _ = next(&mut ws_hub_rx).await;
    let _ = next(&mut storage_rx).await;

    let got = list(&posts_tx).await;
    let ids: HashSet<PostId> = got.into_iter().map(|(id, _)| id).collect();

    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));

    drop(posts_tx);
    let _ = task.await;
}