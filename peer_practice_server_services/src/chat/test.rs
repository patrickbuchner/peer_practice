use crate::chat::*;
use crate::test_utils::{expect_no_message, recv_timeout};
use peer_practice_messages::current::user::UserId;
use test_case::test_case;
use tokio::task::JoinHandle;

async fn arrange_empty() -> (
    mpsc::Sender<ChatMsg>,
    mpsc::Receiver<StorageMsg>,
    mpsc::Receiver<WsHubMsg>,
    JoinHandle<()>,
) {
    let (storage_tx, mut storage_rx) = mpsc::channel::<StorageMsg>(16);
    let (ws_hub_tx, ws_hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let (chat_tx, chat_rx) = mpsc::channel::<ChatMsg>(16);

    let task = tokio::spawn(handle_chats(storage_tx, ws_hub_tx, chat_rx));

    if let StorageMsg::RetrieveChats { respond_to } = recv_timeout(&mut storage_rx).await {
        let _ = respond_to.send(HashMap::new());
    } else {
        panic!("expected RetrieveChats");
    }

    (chat_tx, storage_rx, ws_hub_rx, task)
}

async fn create_chat_for_post(
    chat_tx: &mpsc::Sender<ChatMsg>,
    storage_rx: &mut mpsc::Receiver<StorageMsg>,
    post_id: PostId,
) -> ChatId {
    chat_tx.send(ChatMsg::CreateForPost(post_id)).await.unwrap();
    match recv_timeout(storage_rx).await {
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
        timestamp: chrono::Utc::now(),
    }]
}

fn msgs_three(chat_id: ChatId) -> Vec<Message> {
    vec![
        Message {
            sender: UserId::default(),
            message: "a".to_string(),
            chat_id,
            timestamp: chrono::Utc::now(),
        },
        Message {
            sender: UserId::default(),
            message: "b".to_string(),
            chat_id,
            timestamp: chrono::Utc::now(),
        },
        Message {
            sender: UserId::default(),
            message: "c".to_string(),
            chat_id,
            timestamp: chrono::Utc::now(),
        },
    ]
}

#[test_case(msgs_single ; "one message")]
#[test_case(msgs_three  ; "three messages")]
#[tokio::test]
async fn store_msg_broadcasts_and_persists(builder: MsgBuilder) {
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let post_id = PostId::default();
    let chat_id = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;

    let messages = builder(chat_id);
    let expected_texts: Vec<String> = messages.iter().map(|m| m.message.clone()).collect();

    for m in messages {
        chat_tx.send(ChatMsg::StoreMsg(m)).await.unwrap();
    }

    let mut last_snapshot: Option<HashMap<ChatId, Progress>> = None;
    for _ in 0..expected_texts.len() {
        match recv_timeout(&mut ws_hub_rx).await {
            WsHubMsg::BroadcastAll(ServerToClient::Chat(ChatAction::MessageSent(_))) => {}
            other => panic!("unexpected ws msg: {other:?}"),
        }
        match recv_timeout(&mut storage_rx).await {
            StorageMsg::SaveChats(snapshot) => last_snapshot = Some(snapshot),
            other => panic!("unexpected storage msg: {other:?}"),
        }
    }

    let saved = last_snapshot.unwrap();
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
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let post_id = PostId::default();

    if !first_time {
        let _ = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    }

    chat_tx.send(ChatMsg::CreateForPost(post_id)).await.unwrap();

    if first_time {
        match recv_timeout(&mut storage_rx).await {
            StorageMsg::SaveChats(snapshot) => assert_eq!(1, snapshot.len()),
            other => panic!("expected SaveChats, got {other:?}"),
        }
    } else {
        expect_no_message(&mut storage_rx).await;
    }

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "missing post => Err")]
#[test_case(true  ; "existing post => Ok")]
#[tokio::test]
async fn get_chat_for_post_returns_expected(found: bool) {
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let post_id = PostId::default();

    if found {
        let _ = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    }

    let res = get_for_post(&chat_tx, post_id).await;
    assert_eq!(found, res.is_ok());

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "missing chat_id => Err")]
#[test_case(true  ; "existing chat_id => Ok")]
#[tokio::test]
async fn get_chat_by_id_returns_expected(found: bool) {
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let post_id = PostId::default();
    let chat_id = if found {
        create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await
    } else {
        ChatId::new()
    };

    let res = get_by_id(&chat_tx, chat_id).await;
    assert_eq!(found, res.is_ok());

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "deleting missing chat is a no-op (no save)")]
#[test_case(true  ; "deleting existing chat removes it and persists")]
#[tokio::test]
async fn delete_removes_and_persists(known: bool) {
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let post_id = PostId::default();
    let chat_id = if known {
        create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await
    } else {
        ChatId::new()
    };

    chat_tx.send(ChatMsg::Delete(chat_id)).await.unwrap();

    if known {
        match recv_timeout(&mut storage_rx).await {
            StorageMsg::SaveChats(snapshot) => {
                assert!(
                    !snapshot.contains_key(&chat_id),
                    "deleted chat should not be present in saved snapshot"
                );
            }
            other => panic!("expected SaveChats after Delete, got {other:?}"),
        }

        let res = get_by_id(&chat_tx, chat_id).await;
        assert!(res.is_err(), "deleted chat should not be retrievable");
    } else {
        expect_no_message(&mut storage_rx).await;
    }

    drop(chat_tx);
    let _ = task.await;
}

#[test_case(false ; "missing post => no-op (no save)")]
#[test_case(true  ; "existing post => removes chat and persists")]
#[tokio::test]
async fn delete_for_post_removes_and_persists(known: bool) {
    let (chat_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let post_id = PostId::default();

    if known {
        let _ = create_chat_for_post(&chat_tx, &mut storage_rx, post_id).await;
    }

    chat_tx
        .send(ChatMsg::DeleteForPost(post_id))
        .await
        .unwrap();

    if known {
        match recv_timeout(&mut storage_rx).await {
            StorageMsg::SaveChats(snapshot) => {
                assert!(
                    snapshot.values().all(|progress| progress.post_id != post_id),
                    "deleted chat should not be present in saved snapshot"
                );
            }
            other => panic!("expected SaveChats after DeleteForPost, got {other:?}"),
        }

        let res = get_for_post(&chat_tx, post_id).await;
        assert!(res.is_err(), "deleted chat should not be retrievable");
    } else {
        expect_no_message(&mut storage_rx).await;
    }

    drop(chat_tx);
    let _ = task.await;
}

#[tokio::test]
async fn store_msg_unknown_chat_is_noop() {
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let chat_id = ChatId::new();
    chat_tx
        .send(ChatMsg::StoreMsg(Message {
            sender: UserId::default(),
            message: "missing".to_string(),
            chat_id,
            timestamp: chrono::Utc::now(),
        }))
        .await
        .unwrap();

    expect_no_message(&mut ws_hub_rx).await;
    expect_no_message(&mut storage_rx).await;

    drop(chat_tx);
    let _ = task.await;
}

#[tokio::test]
async fn store_msg_for_missing_post_is_noop() {
    let (chat_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    chat_tx
        .send(ChatMsg::StoreMsgForPost {
            post_id: PostId::new(),
            sender: UserId::default(),
            message: "missing".to_string(),
        })
        .await
        .unwrap();

    expect_no_message(&mut ws_hub_rx).await;
    expect_no_message(&mut storage_rx).await;

    drop(chat_tx);
    let _ = task.await;
}
