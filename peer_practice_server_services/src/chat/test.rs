use crate::chat::*;
use crate::clock::ManualClock;
use chrono::TimeZone;
use peer_practice_messages::current::user::UserId;
use tokio::sync::mpsc::error::TryRecvError;
use std::sync::Arc;
use test_case::test_case;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

fn test_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}

async fn arrange_empty() -> (
    mpsc::Sender<ChatMsg>,
    mpsc::Receiver<StorageMsg>,
    mpsc::Receiver<WsHubMsg>,
    JoinHandle<()>,
) {
    let (storage_tx, mut storage_rx) = mpsc::channel::<StorageMsg>(16);
    let (ws_hub_tx, ws_hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let (chat_tx, chat_rx) = mpsc::channel::<ChatMsg>(16);

    let clock = Arc::new(ManualClock::new(test_timestamp()));
    let task = tokio::spawn(handle_chats(storage_tx, ws_hub_tx, clock, chat_rx));

    if let StorageMsg::RetrieveChats { respond_to } = recv_msg(&mut storage_rx).await {
        let _ = respond_to.send(HashMap::new());
    } else {
        panic!("expected RetrieveChats");
    }

    (chat_tx, storage_rx, ws_hub_rx, task)
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

async fn ping(chat_tx: &mpsc::Sender<ChatMsg>) {
    let (respond_to, recv) = oneshot::channel();
    chat_tx
        .send(ChatMsg::Ping(respond_to))
        .await
        .unwrap();
    recv_oneshot(recv).await
}

async fn create_chat_for_post(
    chat_tx: &mpsc::Sender<ChatMsg>,
    storage_rx: &mut mpsc::Receiver<StorageMsg>,
    post_id: PostId,
) -> ChatId {
    chat_tx.send(ChatMsg::CreateForPost(post_id)).await.unwrap();
    match recv_msg(storage_rx).await {
        StorageMsg::SaveChats(snapshot) => {
            assert_eq!(
                1,
                snapshot.len(),
                "CreateForPost should save exactly one chat"
            );
        }
        other => panic!("expected SaveChats after CreateForPost, got {other:?}"),
    }

    let (respond_to, recv) = oneshot::channel();
    chat_tx
        .send(ChatMsg::GetChatForPost(post_id, respond_to))
        .await
        .unwrap();
    recv.await.unwrap().unwrap().chat_id
}

async fn get_for_post(chat_tx: &mpsc::Sender<ChatMsg>, post_id: PostId) -> Result<Progress, ()> {
    let (respond_to, recv) = oneshot::channel();
    chat_tx
        .send(ChatMsg::GetChatForPost(post_id, respond_to))
        .await
        .unwrap();
    recv.await.unwrap()
}

async fn get_by_id(chat_tx: &mpsc::Sender<ChatMsg>, chat_id: ChatId) -> Result<Progress, ()> {
    let (respond_to, recv) = oneshot::channel();
    chat_tx
        .send(ChatMsg::GetChat(chat_id, respond_to))
        .await
        .unwrap();
    recv.await.unwrap()
}

type MsgBuilder = fn(ChatId) -> Vec<Message>;

fn msgs_single(chat_id: ChatId) -> Vec<Message> {
    vec![Message {
        sender: UserId::default(),
        message: "one".to_string(),
        chat_id,
        timestamp: test_timestamp(),
    }]
}

fn msgs_three(chat_id: ChatId) -> Vec<Message> {
    vec![
        Message {
            sender: UserId::default(),
            message: "a".to_string(),
            chat_id,
            timestamp: test_timestamp(),
        },
        Message {
            sender: UserId::default(),
            message: "b".to_string(),
            chat_id,
            timestamp: test_timestamp(),
        },
        Message {
            sender: UserId::default(),
            message: "c".to_string(),
            chat_id,
            timestamp: test_timestamp(),
        },
    ]
}

#[test_case(msgs_single ; "one message")]
#[test_case(msgs_three  ; "three messages")]
#[tokio::test]
async fn store_msg_broadcasts_and_persists(builder: MsgBuilder) {
    // Arrange
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::default();
    let chat_id = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;

    let messages = builder(chat_id);
    let expected_texts: Vec<String> = messages.iter().map(|m| m.message.clone()).collect();

    // Act
    for m in messages {
        chat_tx.send(ChatMsg::StoreMsg(m)).await.unwrap();
    }

    let mut ws_msgs = Vec::new();
    let mut last_snapshot: Option<HashMap<ChatId, Progress>> = None;
    for _ in 0..expected_texts.len() {
        ws_msgs.push(recv_msg(&mut ws_hub_rx).await);
        let storage_msg = recv_msg(&mut storage_rx).await;
        if let StorageMsg::SaveChats(snapshot) = storage_msg {
            last_snapshot = Some(snapshot);
        }
    }

    // Assert
    for msg in ws_msgs {
        assert!(matches!(
            msg,
            WsHubMsg::BroadcastAll(ServerToClient::Chat(ChatAction::MessageSent(_)))
        ));
    }
    let saved = last_snapshot.expect("SaveChats should be emitted");
    let saved_progress = saved.get(&chat_id).unwrap();
    let saved_texts: Vec<String> = saved_progress
        .content
        .iter()
        .map(|m| m.message.clone())
        .collect();
    assert_eq!(expected_texts, saved_texts);

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(true  ; "first call creates and persists")]
#[test_case(false ; "second call is idempotent (no extra save)")]
#[tokio::test]
async fn create_for_post_is_idempotent(first_time: bool) {
    // Arrange
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::default();

    if !first_time {
        let _ = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    }

    // Act
    chat_tx.send(ChatMsg::CreateForPost(post_id)).await.unwrap();
    let saved = if first_time {
        Some(recv_msg(&mut storage_rx).await)
    } else {
        None
    };

    // Assert
    if first_time {
        match saved.expect("SaveChats should be emitted") {
            StorageMsg::SaveChats(snapshot) => assert_eq!(1, snapshot.len()),
            other => panic!("expected SaveChats, got {other:?}"),
        }
    } else {
        ping(&chat_tx).await;
        assert_empty(&mut storage_rx);
    }

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "missing post => Err")]
#[test_case(true  ; "existing post => Ok")]
#[tokio::test]
async fn get_chat_for_post_returns_expected(found: bool) {
    // Arrange
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::default();

    if found {
        let _ = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    }

    // Act
    let res = get_for_post(&chat_tx, post_id).await;

    // Assert
    assert_eq!(found, res.is_ok());

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "missing chat_id => Err")]
#[test_case(true  ; "existing chat_id => Ok")]
#[tokio::test]
async fn get_chat_by_id_returns_expected(found: bool) {
    // Arrange
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::default();
    let chat_id = if found {
        create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await
    } else {
        ChatId::new()
    };

    // Act
    let res = get_by_id(&chat_tx, chat_id).await;

    // Assert
    assert_eq!(found, res.is_ok());

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "deleting missing chat is a no-op (no save)")]
#[test_case(true  ; "deleting existing chat removes it and persists")]
#[tokio::test]
async fn delete_removes_and_persists(known: bool) {
    // Arrange
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::default();
    let chat_id = if known {
        create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await
    } else {
        ChatId::new()
    };

    // Act
    chat_tx.send(ChatMsg::Delete(chat_id)).await.unwrap();
    let saved = if known {
        Some(recv_msg(&mut storage_rx).await)
    } else {
        None
    };
    let res = if known {
        Some(get_by_id(&chat_tx, chat_id).await)
    } else {
        None
    };

    // Assert
    if known {
        match saved.expect("SaveChats should be emitted") {
            StorageMsg::SaveChats(snapshot) => {
                assert!(
                    !snapshot.contains_key(&chat_id),
                    "deleted chat should not be present in saved snapshot"
                );
            }
            other => panic!("expected SaveChats after Delete, got {other:?}"),
        }

        let res = res.expect("get_by_id should run");
        assert!(res.is_err(), "deleted chat should not be retrievable");
    } else {
        ping(&chat_tx).await;
        assert_empty(&mut storage_rx);
    }

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "missing post => no-op (no save)")]
#[test_case(true  ; "existing post => removes chat and persists")]
#[tokio::test]
async fn delete_for_post_removes_and_persists(known: bool) {
    // Arrange
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::default();

    if known {
        let _ = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    }

    // Act
    chat_tx.send(ChatMsg::DeleteForPost(post_id)).await.unwrap();
    let saved = if known {
        Some(recv_msg(&mut storage_rx).await)
    } else {
        None
    };
    let res = if known {
        Some(get_for_post(&chat_tx, post_id).await)
    } else {
        None
    };

    // Assert
    if known {
        match saved.expect("SaveChats should be emitted") {
            StorageMsg::SaveChats(snapshot) => {
                assert!(
                    snapshot
                        .values()
                        .all(|progress| progress.post_id != post_id),
                    "deleted chat should not be present in saved snapshot"
                );
            }
            other => panic!("expected SaveChats after DeleteForPost, got {other:?}"),
        }

        let res = res.expect("get_for_post should run");
        assert!(res.is_err(), "deleted chat should not be retrievable");
    } else {
        ping(&chat_tx).await;
        assert_empty(&mut storage_rx);
    }

    drop(chat_tx);
    let _ = task.await;
}

#[tokio::test]
async fn store_msg_unknown_chat_is_noop() {
    // Arrange
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let chat_id = ChatId::new();

    // Act
    chat_tx
        .send(ChatMsg::StoreMsg(Message {
            sender: UserId::default(),
            message: "missing".to_string(),
            chat_id,
            timestamp: test_timestamp(),
        }))
        .await
        .unwrap();

    // Assert
    ping(&chat_tx).await;
    assert_empty(&mut ws_hub_rx);
    assert_empty(&mut storage_rx);

    drop(chat_tx);
    let _ = task.await;
}

#[tokio::test]
async fn store_msg_for_missing_post_is_noop() {
    // Arrange
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    // Act
    chat_tx
        .send(ChatMsg::StoreMsgForPost {
            post_id: PostId::new(),
            sender: UserId::default(),
            message: "missing".to_string(),
        })
        .await
        .unwrap();

    // Assert
    ping(&chat_tx).await;
    assert_empty(&mut ws_hub_rx);
    assert_empty(&mut storage_rx);

    drop(chat_tx);
    let _ = task.await;
}

#[tokio::test]
async fn store_msg_for_post_broadcasts_and_persists() {
    // Arrange
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;
    ping(&chat_tx).await;

    let post_id = PostId::new();
    let chat_id = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    let sender = UserId::new();
    let message = "hello".to_string();

    // Act
    chat_tx
        .send(ChatMsg::StoreMsgForPost {
            post_id,
            sender,
            message: message.clone(),
        })
        .await
        .unwrap();

    let ws_msg = recv_msg(&mut ws_hub_rx).await;
    let storage_msg = recv_msg(&mut storage_rx).await;

    // Assert
    match ws_msg {
        WsHubMsg::BroadcastAll(ServerToClient::Chat(ChatAction::MessageSent(sent))) => {
            assert_eq!(chat_id, sent.chat_id);
            assert_eq!(sender, sent.sender);
            assert_eq!(message, sent.message);
        }
        other => panic!("expected MessageSent broadcast, got {other:?}"),
    }
    match storage_msg {
        StorageMsg::SaveChats(snapshot) => {
            let saved = snapshot.get(&chat_id).expect("chat should exist");
            assert_eq!(1, saved.content.len());
            assert_eq!(message, saved.content[0].message);
        }
        other => panic!("expected SaveChats, got {other:?}"),
    }

    drop(chat_tx);
    let _ = task.await;
}
