use crate::clock::ManualClock;
use crate::pending_logins::{handle_pending_logins, PendingLoginsMsg};
use crate::test_utils::TEST_TIMEOUT;
use chrono::{Duration, TimeZone};
use peer_practice_messages::current::email::Email;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;

fn test_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}

async fn arrange() -> (
    mpsc::Sender<PendingLoginsMsg>,
    JoinHandle<()>,
    Arc<ManualClock>,
) {
    let (tx, rx) = mpsc::channel::<PendingLoginsMsg>(16);
    let clock = Arc::new(ManualClock::new(test_timestamp()));
    let task = tokio::spawn(handle_pending_logins(clock.clone(), rx));
    (tx, task, clock)
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
    let (tx, task, _clock) = arrange().await;

    let address = Email::new("missing@example.com").unwrap();
    let got = get_code(&tx, address).await;

    assert_eq!(None, got);

    drop(tx);
    let _ = task.await;
}

#[tokio::test]
async fn upsert_then_get_returns_code() {
    let (tx, task, _clock) = arrange().await;

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
    let (tx, task, _clock) = arrange().await;

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
    let (tx, task, _clock) = arrange().await;

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

#[tokio::test]
async fn expired_entry_returns_none_without_sleeping() {
    let (tx, task, clock) = arrange().await;

    let address = Email::new("expired@example.com").unwrap();
    tx.send(PendingLoginsMsg::Upsert {
        address: address.clone(),
        code: 333_333,
    })
    .await
    .unwrap();

    let got = get_code(&tx, address.clone()).await;
    assert_eq!(Some(333_333), got);

    clock.advance(Duration::minutes(16));

    let got = get_code(&tx, address).await;
    assert_eq!(None, got);

    drop(tx);
    let _ = task.await;
}
