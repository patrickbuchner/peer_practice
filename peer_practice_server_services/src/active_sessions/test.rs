use crate::active_sessions::{ActiveSessionsMsg, handle_active_sessions};
use crate::clock::ManualClock;
use crate::storage::StorageMsg;
use chrono::{Duration, TimeZone};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

fn test_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}

async fn arrange(
    jwt_expiry_duration: Duration,
) -> (
    mpsc::Sender<ActiveSessionsMsg>,
    mpsc::Receiver<StorageMsg>,
    JoinHandle<()>,
    Arc<ManualClock>,
) {
    let (active_tx, active_rx) = mpsc::channel::<ActiveSessionsMsg>(16);
    let (storage_tx, mut storage_rx) = mpsc::channel::<StorageMsg>(16);

    let clock = Arc::new(ManualClock::new(test_timestamp()));
    let task = tokio::spawn(handle_active_sessions(
        jwt_expiry_duration,
        clock.clone(),
        storage_tx,
        active_rx,
    ));

    // handle initial snapshot load
    match storage_rx.recv().await.expect("storage channel closed") {
        StorageMsg::LoadJson {
            namespace,
            respond_to,
        } => {
            assert_eq!("active_sessions", namespace);
            let _ = respond_to.send(Value::Null);
        }
        other => panic!("expected StorageMsg::LoadJson, got {other:?}"),
    }

    (active_tx, storage_rx, task, clock)
}

async fn validate(active_tx: &mpsc::Sender<ActiveSessionsMsg>, jwt: &str) -> Option<String> {
    let (tx, rx) = oneshot::channel();
    active_tx
        .send(ActiveSessionsMsg::ValidateJwt(jwt.to_string(), tx))
        .await
        .unwrap();
    rx.await.expect("oneshot closed")
}

#[tokio::test]
async fn validate_allows_when_not_invalidated() {
    let (active_tx, _storage_rx, task, _clock) = arrange(Duration::days(30)).await;

    let jwt = "jwt-a";
    let got = validate(&active_tx, jwt).await;

    assert_eq!(Some(jwt.to_string()), got);

    drop(active_tx);
    let _ = task.await;
}

#[tokio::test]
async fn validate_allows_within_5_min_grace_period() {
    let (active_tx, _storage_rx, task, clock) = arrange(Duration::days(30)).await;

    let jwt = "jwt-b";

    active_tx
        .send(ActiveSessionsMsg::InvalidateJwt(jwt.to_string()))
        .await
        .unwrap();

    let (tx, rx) = oneshot::channel();
    active_tx.send(ActiveSessionsMsg::Ping(tx)).await.unwrap();
    rx.await.expect("oneshot closed");

    // Within grace (<= 5 minutes)
    clock.advance(Duration::minutes(4) + Duration::seconds(59));

    let got = validate(&active_tx, jwt).await;
    assert_eq!(Some(jwt.to_string()), got);

    drop(active_tx);
    let _ = task.await;
}

#[tokio::test]
async fn validate_rejects_after_5_min_grace_period() {
    let (active_tx, _storage_rx, task, clock) = arrange(Duration::days(30)).await;

    let jwt = "jwt-c";

    active_tx
        .send(ActiveSessionsMsg::InvalidateJwt(jwt.to_string()))
        .await
        .unwrap();

    let (tx, rx) = oneshot::channel();
    active_tx.send(ActiveSessionsMsg::Ping(tx)).await.unwrap();
    rx.await.expect("oneshot closed");

    // Past grace (> 5 minutes)
    clock.advance(Duration::minutes(10) + Duration::seconds(1));

    let got = validate(&active_tx, jwt).await;
    assert_eq!(None, got);

    drop(active_tx);
    let _ = task.await;
}
