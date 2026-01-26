use super::{ConnectionHandle, ConnectionId, WsHubMsg, handle_ws_hub_actions};
use crate::test_utils::{expect_no_message_unbounded, recv_timeout_unbounded, TEST_TIMEOUT};
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::user::UserId;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;

async fn arrange() -> (mpsc::Sender<WsHubMsg>, JoinHandle<()>) {
    let (hub_tx, hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let task = tokio::spawn(handle_ws_hub_actions(hub_rx, hub_tx.clone()));
    (hub_tx, task)
}

async fn join(
    hub_tx: &mpsc::Sender<WsHubMsg>,
    user_id: UserId,
) -> (
    ConnectionHandle,
    tokio::sync::mpsc::UnboundedReceiver<ServerToClient>,
) {
    let (respond_to, recv) = oneshot::channel();
    hub_tx
        .send(WsHubMsg::Join {
            user_id,
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
async fn join_returns_handle_and_receiver() {
    let (hub_tx, task) = arrange().await;

    let user_id = UserId::default();
    let (_handle, mut rx) = join(&hub_tx, user_id).await;

    hub_tx
        .send(WsHubMsg::BroadcastUser {
            user_id,
            msg: ServerToClient::MessageNotYetKnown,
        })
        .await
        .unwrap();

    let received = recv_timeout_unbounded(&mut rx).await;
    assert!(matches!(received, Some(ServerToClient::MessageNotYetKnown)));

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn broadcast_all_delivers_to_all_connections() {
    let (hub_tx, task) = arrange().await;

    let user1 = UserId::default();
    let user2 = UserId::new();

    let (_h1, mut rx1) = join(&hub_tx, user1).await;
    let (_h2, mut rx2) = join(&hub_tx, user2).await;

    hub_tx
        .send(WsHubMsg::BroadcastAll(ServerToClient::MessageNotYetKnown))
        .await
        .unwrap();

    assert!(matches!(
        recv_timeout_unbounded(&mut rx1).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));
    assert!(matches!(
        recv_timeout_unbounded(&mut rx2).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn broadcast_user_only_delivers_to_that_user() {
    let (hub_tx, task) = arrange().await;

    let user1 = UserId::default();
    let user2 = UserId::new();

    let (_h1, mut rx1) = join(&hub_tx, user1).await;
    let (_h2, mut rx2) = join(&hub_tx, user2).await;

    hub_tx
        .send(WsHubMsg::BroadcastUser {
            user_id: user1,
            msg: ServerToClient::MessageNotYetKnown,
        })
        .await
        .unwrap();

    assert!(matches!(
        recv_timeout_unbounded(&mut rx1).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));

    expect_no_message_unbounded(&mut rx2).await;

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn send_delivers_only_to_specific_connection() {
    let (hub_tx, task) = arrange().await;

    let user = UserId::default();

    let (h1, mut rx1) = join(&hub_tx, user).await;
    let (h2, mut rx2) = join(&hub_tx, user).await;

    let id1: ConnectionId = h1.id();
    let id2: ConnectionId = h2.id();

    hub_tx
        .send(WsHubMsg::Send {
            user_id: user,
            con_id: id1,
            msg: ServerToClient::MessageNotYetKnown,
        })
        .await
        .unwrap();

    assert!(matches!(
        recv_timeout_unbounded(&mut rx1).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));

    expect_no_message_unbounded(&mut rx2).await;

    // sanity: sending to the other id should reach rx2
    hub_tx
        .send(WsHubMsg::Send {
            user_id: user,
            con_id: id2,
            msg: ServerToClient::MessageNotYetKnown,
        })
        .await
        .unwrap();

    assert!(matches!(
        recv_timeout_unbounded(&mut rx2).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn dropping_handle_leaves_and_closes_receiver() {
    let (hub_tx, task) = arrange().await;

    let user = UserId::default();

    let (handle, mut rx) = join(&hub_tx, user).await;
    drop(handle); // triggers Leave via Drop

    // after Leave, the hub drops the sender, so the receiver should close
    let got = recv_timeout_unbounded(&mut rx).await;
    assert!(got.is_none());

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}
