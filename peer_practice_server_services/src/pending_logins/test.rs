use crate::pending_logins::{PendingLoginsMsg, handle_pending_logins};
use crate::test_utils::TEST_TIMEOUT;
use peer_practice_messages::current::email::Email;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;

async fn arrange() -> (mpsc::Sender<PendingLoginsMsg>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<PendingLoginsMsg>(16);
    let task = tokio::spawn(handle_pending_logins(rx));
    (tx, task)
}

async fn get_code(tx: &mpsc::Sender<PendingLoginsMsg>, address: Email) -> Option<u32> {
    let (respond_to, recv) = oneshot::channel();
    tx.send(PendingLoginsMsg::GetByAddress {
        address,
        respond_to,
    })
    .await
    .unwrap();

    timeout(TEST_TIMEOUT, recv)
        .await
        .expect("timed out")
        .expect("oneshot closed")
}

#[tokio::test]
async fn get_missing_returns_none() {
    let (tx, task) = arrange().await;

    let address = Email::new("missing@example.com").unwrap();
    let got = get_code(&tx, address).await;

    assert_eq!(None, got);

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn upsert_then_get_returns_code() {
    let (tx, task) = arrange().await;

    let address = Email::new("user@example.com").unwrap();
    tx.send(PendingLoginsMsg::Upsert {
        address: address.clone(),
        code: 123_456,
    })
    .await
    .unwrap();

    let got = get_code(&tx, address).await;
    assert_eq!(Some(123_456), got);

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn upsert_overwrites_existing_code() {
    let (tx, task) = arrange().await;

    let address = Email::new("overwrite@example.com").unwrap();

    tx.send(PendingLoginsMsg::Upsert {
        address: address.clone(),
        code: 111_111,
    })
    .await
    .unwrap();

    tx.send(PendingLoginsMsg::Upsert {
        address: address.clone(),
        code: 222_222,
    })
    .await
    .unwrap();

    let got = get_code(&tx, address).await;
    assert_eq!(Some(222_222), got);

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn remove_deletes_entry() {
    let (tx, task) = arrange().await;

    let address = Email::new("remove@example.com").unwrap();

    tx.send(PendingLoginsMsg::Upsert {
        address: address.clone(),
        code: 999_999,
    })
    .await
    .unwrap();

    tx.send(PendingLoginsMsg::Remove {
        address: address.clone(),
    })
    .await
    .unwrap();

    let got = get_code(&tx, address).await;
    assert_eq!(None, got);

    drop(tx);
    let _ = task.await;
}
