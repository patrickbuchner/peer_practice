use super::{UsersMsg, handle_user_actions};
use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::email::Email;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::server_to_client::UserAction;
use peer_practice_messages::current::user::{User, UserId};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{Duration, timeout};

async fn next<T>(rx: &mut mpsc::Receiver<T>) -> T {
    timeout(Duration::from_millis(300), rx.recv())
        .await
        .expect("timed out")
        .expect("channel closed")
}

async fn arrange_empty() -> (
    mpsc::Sender<UsersMsg>,
    mpsc::Receiver<StorageMsg>,
    mpsc::Receiver<WsHubMsg>,
    JoinHandle<()>,
) {
    let (storage_tx, mut storage_rx) = mpsc::channel::<StorageMsg>(16);
    let (ws_hub_tx, ws_hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let (users_tx, users_rx) = mpsc::channel::<UsersMsg>(16);

    let task = tokio::spawn(handle_user_actions(storage_tx, ws_hub_tx, users_rx));

    if let StorageMsg::RetrieveUsers { respond_to } = next(&mut storage_rx).await {
        let _ = respond_to.send(HashMap::new());
    } else {
        panic!("expected RetrieveUsers");
    }

    (users_tx, storage_rx, ws_hub_rx, task)
}

async fn get_by_email(users_tx: &mpsc::Sender<UsersMsg>, email: Email) -> Option<UserId> {
    let (respond_to, recv) = oneshot::channel();
    users_tx
        .send(UsersMsg::GetByEmail { email, respond_to })
        .await
        .unwrap();

    timeout(Duration::from_millis(300), recv)
        .await
        .expect("timed out")
        .expect("oneshot closed")
}

async fn get_by_id(users_tx: &mpsc::Sender<UsersMsg>, id: UserId) -> Option<User> {
    let (respond_to, recv) = oneshot::channel();
    users_tx
        .send(UsersMsg::GetById { id, respond_to })
        .await
        .unwrap();

    timeout(Duration::from_millis(300), recv)
        .await
        .expect("timed out")
        .expect("oneshot closed")
}

#[tokio::test]
async fn get_by_id_missing_returns_none() {
    let (users_tx, _storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let missing = UserId::new();
    let got = get_by_id(&users_tx, missing).await;

    assert!(got.is_none());

    drop(users_tx);
    let _ = task.await;
}

#[tokio::test]
async fn get_by_email_creates_user_persists_and_is_idempotent() {
    let (users_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let email = Email::new("user@example.com").unwrap();

    let id1 = get_by_email(&users_tx, email.clone())
        .await
        .expect("should create a new user id");

    match next(&mut storage_rx).await {
        StorageMsg::SaveUsers(snapshot) => {
            assert!(
                snapshot.contains_key(&id1),
                "saved snapshot should include created user"
            );
        }
        other => panic!("expected SaveUsers after first GetByEmail, got {other:?}"),
    }

    let id2 = get_by_email(&users_tx, email.clone()).await.unwrap();
    assert_eq!(id1, id2, "second GetByEmail should return same id");

    let got = timeout(Duration::from_millis(150), storage_rx.recv()).await;
    assert!(
        got.is_err(),
        "expected no storage write on cached GetByEmail"
    );

    drop(users_tx);
    let _ = task.await;
}

#[tokio::test]
async fn update_persists_broadcasts_and_get_by_id_returns_updated_user() {
    let (users_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let email = Email::new("update@example.com").unwrap();
    let id = get_by_email(&users_tx, email.clone()).await.unwrap();

    // consume SaveUsers from the create-on-demand in GetByEmail
    let _ = next(&mut storage_rx).await;

    let updated = User {
        id,
        email: email.clone(),
        display_name: Some("Tester".to_string()),
    };

    users_tx
        .send(UsersMsg::Update {
            id,
            user: updated.clone(),
        })
        .await
        .unwrap();

    match next(&mut storage_rx).await {
        StorageMsg::SaveUsers(snapshot) => {
            let saved = snapshot.get(&id).expect("updated user should be saved");
            assert_eq!(updated.display_name, saved.display_name);
            assert_eq!(updated.email.value(), saved.email.value());
        }
        other => panic!("expected SaveUsers after Update, got {other:?}"),
    }

    match next(&mut ws_hub_rx).await {
        WsHubMsg::BroadcastAll(ServerToClient::User(UserAction::User(got_id, _))) => {
            assert_eq!(id, got_id);
        }
        other => panic!("expected User broadcast after Update, got {other:?}"),
    }

    let got = get_by_id(&users_tx, id).await.expect("user should exist");
    assert_eq!(updated.display_name, got.display_name);
    assert_eq!(updated.email.value(), got.email.value());

    drop(users_tx);
    let _ = task.await;
}

#[tokio::test]
async fn update_with_changed_email_updates_email_index() {
    let (users_tx, mut storage_rx, mut ws_hub_rx, task) = arrange_empty().await;

    let old_email = Email::new("old@example.com").unwrap();
    let new_email = Email::new("new@example.com").unwrap();

    let id1 = get_by_email(&users_tx, old_email.clone()).await.unwrap();
    let _ = next(&mut storage_rx).await; // SaveUsers from create

    users_tx
        .send(UsersMsg::Update {
            id: id1,
            user: User {
                id: id1,
                email: new_email.clone(),
                display_name: Some("Renamed".to_string()),
            },
        })
        .await
        .unwrap();

    let _ = next(&mut storage_rx).await; // SaveUsers from update
    let _ = next(&mut ws_hub_rx).await; // BroadcastAll from update

    // old email should no longer resolve to id1; it will create a *new* user/id
    let id2 = get_by_email(&users_tx, old_email.clone()).await.unwrap();
    assert_ne!(
        id1, id2,
        "old email should no longer map to the updated user id"
    );

    match next(&mut storage_rx).await {
        StorageMsg::SaveUsers(snapshot) => {
            assert!(snapshot.contains_key(&id2));
        }
        other => panic!("expected SaveUsers after recreating old email, got {other:?}"),
    }

    // new email should still map to id1 and not trigger another SaveUsers
    let got = get_by_email(&users_tx, new_email.clone()).await.unwrap();
    assert_eq!(id1, got);

    let got = timeout(Duration::from_millis(150), storage_rx.recv()).await;
    assert!(
        got.is_err(),
        "expected no storage write for cached new-email lookup"
    );

    drop(users_tx);
    let _ = task.await;
}

#[tokio::test]
async fn remove_deletes_user_persists_and_email_can_be_recreated() {
    let (users_tx, mut storage_rx, _ws_hub_rx, task) = arrange_empty().await;

    let email = Email::new("remove@example.com").unwrap();
    let id1 = get_by_email(&users_tx, email.clone()).await.unwrap();
    let _ = next(&mut storage_rx).await; // SaveUsers from create

    users_tx.send(UsersMsg::Remove { id: id1 }).await.unwrap();

    match next(&mut storage_rx).await {
        StorageMsg::SaveUsers(snapshot) => {
            assert!(
                !snapshot.contains_key(&id1),
                "removed user should not be present in saved snapshot"
            );
        }
        other => panic!("expected SaveUsers after Remove, got {other:?}"),
    }

    assert!(get_by_id(&users_tx, id1).await.is_none());

    let id2 = get_by_email(&users_tx, email.clone()).await.unwrap();
    assert_ne!(
        id1, id2,
        "removing should allow re-creating the email with a new id"
    );

    match next(&mut storage_rx).await {
        StorageMsg::SaveUsers(snapshot) => {
            assert!(snapshot.contains_key(&id2));
        }
        other => panic!("expected SaveUsers after re-create, got {other:?}"),
    }

    drop(users_tx);
    let _ = task.await;
}
