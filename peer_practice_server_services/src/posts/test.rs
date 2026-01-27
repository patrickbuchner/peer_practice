use super::{PostsMsg, handle_posts};
use crate::chat::ChatMsg;
use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::level::Level;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::server_to_client::PostAction;
use peer_practice_messages::current::post::{Post, PostId, Topics};
use peer_practice_messages::current::user::UserId;
use peer_practice_messages::test_helpers_impl::fixed_timestamp;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

async fn arrange_empty() -> (
    mpsc::Sender<PostsMsg>,
    mpsc::Receiver<StorageMsg>,
    mpsc::Receiver<WsHubMsg>,
    mpsc::Receiver<ChatMsg>,
    JoinHandle<()>,
) {
    let (storage_tx, mut storage_rx) = mpsc::channel::<StorageMsg>(16);
    let (ws_hub_tx, ws_hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let (chat_tx, chat_rx) = mpsc::channel::<ChatMsg>(16);
    let (posts_tx, posts_rx) = mpsc::channel::<PostsMsg>(16);

    let task = tokio::spawn(handle_posts(storage_tx, ws_hub_tx, chat_tx, posts_rx));

    if let StorageMsg::RetrievePosts { respond_to } = recv_msg(&mut storage_rx).await {
        let _ = respond_to.send(HashMap::new());
    } else {
        panic!("expected RetrievePosts");
    }

    (posts_tx, storage_rx, ws_hub_rx, chat_rx, task)
}

async fn recv_msg<T>(rx: &mut mpsc::Receiver<T>) -> T {
    match rx.recv().await {
        Some(msg) => msg,
        None => panic!("channel closed"),
    }
}

async fn recv_oneshot<T>(rx: oneshot::Receiver<T>) -> T {
    rx.await.expect("oneshot closed")
}

fn assert_empty<T>(rx: &mut mpsc::Receiver<T>) {
    match rx.try_recv() {
        Ok(_) => panic!("expected no message"),
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => panic!("channel closed"),
    }
}

async fn ping(posts_tx: &mpsc::Sender<PostsMsg>) {
    let (respond_to, recv) = oneshot::channel();
    posts_tx
        .send(PostsMsg::Ping(respond_to))
        .await
        .unwrap();
    recv_oneshot(recv).await
}

fn mk_post(owner: UserId) -> Post {
    Post {
        title: Topics::default(),
        content: "hello".to_string(),
        level: Level::Beginner1,
        owner,
        date: fixed_timestamp(),
        partaking_users: HashSet::new(),
    }
}

async fn get(posts_tx: &mpsc::Sender<PostsMsg>, post_id: PostId) -> Option<Post> {
    let (respond_to, recv) = oneshot::channel();
    posts_tx
        .send(PostsMsg::Get(post_id, respond_to))
        .await
        .unwrap();
    recv.await.unwrap()
}

async fn list(posts_tx: &mpsc::Sender<PostsMsg>) -> Vec<(PostId, Post)> {
    let (respond_to, recv) = oneshot::channel();
    posts_tx.send(PostsMsg::List(respond_to)).await.unwrap();
    recv.await.unwrap()
}

#[tokio::test]
async fn get_missing_returns_none() {
    // Arrange
    let (posts_tx, _storage_rx, _ws_hub_rx, _chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let missing = PostId::new();

    // Act
    let got = get(&posts_tx, missing).await;

    // Assert
    assert!(got.is_none());

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn new_broadcasts_and_persists_and_returns_id() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, mut chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let owner = UserId::default();
    let post = mk_post(owner);

    let (id_tx, id_rx) = oneshot::channel();

    // Act
    posts_tx
        .send(PostsMsg::New(post.clone(), id_tx))
        .await
        .unwrap();
    let id = recv_oneshot(id_rx).await;
    let ws_msg = recv_msg(&mut ws_hub_rx).await;
    let storage_msg = recv_msg(&mut storage_rx).await;
    let chat_msg = recv_msg(&mut chat_rx).await;

    // Assert
    match ws_msg {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, got_post))) => {
            assert_eq!(id, got_id);
            assert_eq!(post.content, got_post.content);
        }
        other => panic!("expected Post broadcast, got {other:?}"),
    }
    match storage_msg {
        StorageMsg::SavePosts(snapshot) => {
            assert!(
                snapshot.contains_key(&id),
                "saved snapshot should contain new post id"
            );
        }
        other => panic!("expected SavePosts, got {other:?}"),
    }
    match chat_msg {
        ChatMsg::CreateForPost(got_id) => assert_eq!(id, got_id),
        other => panic!("expected CreateForPost, got {other:?}"),
    }

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn upsert_then_get_returns_post() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, _chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let id = PostId::new();
    let owner = UserId::default();
    let post = mk_post(owner);

    // Act
    posts_tx
        .send(PostsMsg::Upsert(id, post.clone()))
        .await
        .unwrap();
    let ws_msg = recv_msg(&mut ws_hub_rx).await;
    let storage_msg = recv_msg(&mut storage_rx).await;
    let got = get(&posts_tx, id).await.expect("post should exist");

    // Assert
    match ws_msg {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, _))) => {
            assert_eq!(id, got_id);
        }
        other => panic!("expected Post broadcast, got {other:?}"),
    }
    match storage_msg {
        StorageMsg::SavePosts(snapshot) => assert!(snapshot.contains_key(&id)),
        other => panic!("expected SavePosts, got {other:?}"),
    }
    assert_eq!(post.content, got.content);

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn remove_broadcasts_and_persists() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, mut chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let id = PostId::new();
    let post = mk_post(UserId::default());

    posts_tx.send(PostsMsg::Upsert(id, post)).await.unwrap();
    let _ = recv_msg(&mut ws_hub_rx).await;
    let _ = recv_msg(&mut storage_rx).await;

    // Act
    posts_tx.send(PostsMsg::Remove(id)).await.unwrap();
    let ws_msg = recv_msg(&mut ws_hub_rx).await;
    let storage_msg = recv_msg(&mut storage_rx).await;
    let chat_msg = recv_msg(&mut chat_rx).await;
    let got = get(&posts_tx, id).await;

    // Assert
    match ws_msg {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::RemovedPost(got_id))) => {
            assert_eq!(id, got_id);
        }
        other => panic!("expected RemovedPost broadcast, got {other:?}"),
    }
    match storage_msg {
        StorageMsg::SavePosts(snapshot) => {
            assert!(
                !snapshot.contains_key(&id),
                "deleted post should not be present in saved snapshot"
            );
        }
        other => panic!("expected SavePosts, got {other:?}"),
    }
    match chat_msg {
        ChatMsg::DeleteForPost(got_id) => assert_eq!(id, got_id),
        other => panic!("expected DeleteForPost, got {other:?}"),
    }
    assert!(got.is_none());

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn join_then_leave_updates_partaking_users_and_persists() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, mut chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let id = PostId::new();
    let owner = UserId::default();
    let mut post = mk_post(owner);
    post.partaking_users = HashSet::new();
    posts_tx.send(PostsMsg::Upsert(id, post)).await.unwrap();
    let _ = recv_msg(&mut ws_hub_rx).await;
    let _ = recv_msg(&mut storage_rx).await;

    let user = UserId::default();

    // Act
    posts_tx.send(PostsMsg::UserJoins(id, user)).await.unwrap();
    let ws_join = recv_msg(&mut ws_hub_rx).await;
    let storage_join = recv_msg(&mut storage_rx).await;
    let chat_join = recv_msg(&mut chat_rx).await;
    posts_tx.send(PostsMsg::UserLeaves(id, user)).await.unwrap();
    let ws_leave = recv_msg(&mut ws_hub_rx).await;
    let storage_leave = recv_msg(&mut storage_rx).await;
    let chat_leave = recv_msg(&mut chat_rx).await;

    // Assert
    match ws_join {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, got_post))) => {
            assert_eq!(id, got_id);
            assert!(got_post.partaking_users.contains(&user));
        }
        other => panic!("expected Post broadcast after join, got {other:?}"),
    }
    match storage_join {
        StorageMsg::SavePosts(snapshot) => {
            let saved = snapshot.get(&id).unwrap();
            assert!(saved.partaking_users.contains(&user));
        }
        other => panic!("expected SavePosts after join, got {other:?}"),
    }
    match chat_join {
        ChatMsg::StoreMsgForPost {
            post_id: got_id,
            sender: got_user,
            message,
        } => {
            assert_eq!(id, got_id);
            assert_eq!(user, got_user);
            assert_eq!("joined the post", message);
        }
        other => panic!("expected StoreMsgForPost after join, got {other:?}"),
    }
    match ws_leave {
        WsHubMsg::BroadcastAll(ServerToClient::Post(PostAction::Post(got_id, got_post))) => {
            assert_eq!(id, got_id);
            assert!(!got_post.partaking_users.contains(&user));
        }
        other => panic!("expected Post broadcast after leave, got {other:?}"),
    }
    match storage_leave {
        StorageMsg::SavePosts(snapshot) => {
            let saved = snapshot.get(&id).unwrap();
            assert!(!saved.partaking_users.contains(&user));
        }
        other => panic!("expected SavePosts after leave, got {other:?}"),
    }
    match chat_leave {
        ChatMsg::StoreMsgForPost {
            post_id: got_id,
            sender: got_user,
            message,
        } => {
            assert_eq!(id, got_id);
            assert_eq!(user, got_user);
            assert_eq!("left the post", message);
        }
        other => panic!("expected StoreMsgForPost after leave, got {other:?}"),
    }

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn join_missing_post_is_noop() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, mut chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    // Act
    posts_tx
        .send(PostsMsg::UserJoins(PostId::new(), UserId::new()))
        .await
        .unwrap();

    // Assert
    ping(&posts_tx).await;
    assert_empty(&mut ws_hub_rx);
    assert_empty(&mut storage_rx);
    assert_empty(&mut chat_rx);

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn list_returns_all_posts() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, _chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let id1 = PostId::new();
    let id2 = PostId::new();

    posts_tx
        .send(PostsMsg::Upsert(id1, mk_post(UserId::default())))
        .await
        .unwrap();
    let _ = recv_msg(&mut ws_hub_rx).await;
    let _ = recv_msg(&mut storage_rx).await;

    posts_tx
        .send(PostsMsg::Upsert(id2, mk_post(UserId::default())))
        .await
        .unwrap();
    let _ = recv_msg(&mut ws_hub_rx).await;
    let _ = recv_msg(&mut storage_rx).await;

    // Act
    let got = list(&posts_tx).await;
    let ids: HashSet<PostId> = got.into_iter().map(|(id, _)| id).collect();

    // Assert
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));

    drop(posts_tx);
    let _ = task.await;
}

#[tokio::test]
async fn join_leave_unknown_post_is_noop() {
    // Arrange
    let (posts_tx, mut storage_rx, mut ws_hub_rx, mut chat_rx, task) = arrange_empty().await;
    ping(&posts_tx).await;

    let post_id = PostId::new();
    let user = UserId::new();

    // Act
    posts_tx
        .send(PostsMsg::UserJoins(post_id, user))
        .await
        .unwrap();
    posts_tx
        .send(PostsMsg::UserLeaves(post_id, user))
        .await
        .unwrap();

    // Assert
    ping(&posts_tx).await;
    assert_empty(&mut ws_hub_rx);
    assert_empty(&mut storage_rx);
    assert_empty(&mut chat_rx);

    drop(posts_tx);
    let _ = task.await;
}
