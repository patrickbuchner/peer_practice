use super::{ConnectionHandle, ConnectionId, WsHubMsg, handle_ws_hub_actions};
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::messages::server_to_client::UserAction;
use peer_practice_messages::current::user::UserId;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

async fn arrange() -> (mpsc::Sender<WsHubMsg>, JoinHandle<()>) {
    let (hub_tx, hub_rx) = mpsc::channel::<WsHubMsg>(16);
    let task = tokio::spawn(handle_ws_hub_actions(hub_rx, hub_tx.clone()));
    (hub_tx, task)
}

async fn recv_oneshot<T>(rx: oneshot::Receiver<T>) -> T {
    rx.await.expect("oneshot closed")
}

async fn recv_unbounded<T>(rx: &mut mpsc::UnboundedReceiver<T>) -> Option<T> {
    rx.recv().await
}

fn assert_empty_unbounded<T>(rx: &mut mpsc::UnboundedReceiver<T>) {
    match rx.try_recv() {
        Ok(_) => panic!("expected no message"),
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => panic!("channel closed"),
    }
}

async fn ping(hub_tx: &mpsc::Sender<WsHubMsg>) {
    let (respond_to, recv) = oneshot::channel();
    hub_tx
        .send(WsHubMsg::Ping { respond_to })
        .await
        .unwrap();
    recv_oneshot(recv).await
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

    recv_oneshot(recv).await
}

#[tokio::test]
async fn join_returns_handle_and_receiver() {
    // Arrange
    let (hub_tx, task) = arrange().await;
    ping(&hub_tx).await;

    let user_id = UserId::default();

    // Act
    let (_handle, mut rx) = join(&hub_tx, user_id).await;
    hub_tx
        .send(WsHubMsg::BroadcastUser {
            user_id,
            msg: ServerToClient::MessageNotYetKnown,
        })
        .await
        .unwrap();

    // Assert
    let received = recv_unbounded(&mut rx).await;
    assert!(matches!(received, Some(ServerToClient::MessageNotYetKnown)));

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn broadcast_all_delivers_to_all_connections() {
    // Arrange
    let (hub_tx, task) = arrange().await;
    ping(&hub_tx).await;

    let user1 = UserId::default();
    let user2 = UserId::new();

    let (_h1, mut rx1) = join(&hub_tx, user1).await;
    let (_h2, mut rx2) = join(&hub_tx, user2).await;

    // Act
    hub_tx
        .send(WsHubMsg::BroadcastAll(ServerToClient::MessageNotYetKnown))
        .await
        .unwrap();

    // Assert
    assert!(matches!(
        recv_unbounded(&mut rx1).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));
    assert!(matches!(
        recv_unbounded(&mut rx2).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn broadcast_user_only_delivers_to_that_user() {
    // Arrange
    let (hub_tx, task) = arrange().await;
    ping(&hub_tx).await;

    let user1 = UserId::default();
    let user2 = UserId::new();

    let (_h1, mut rx1) = join(&hub_tx, user1).await;
    let (_h2, mut rx2) = join(&hub_tx, user2).await;

    // Act
    hub_tx
        .send(WsHubMsg::BroadcastUser {
            user_id: user1,
            msg: ServerToClient::MessageNotYetKnown,
        })
        .await
        .unwrap();

    // Assert
    assert!(matches!(
        recv_unbounded(&mut rx1).await,
        Some(ServerToClient::MessageNotYetKnown)
    ));

    ping(&hub_tx).await;
    assert_empty_unbounded(&mut rx2);

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn send_delivers_only_to_specific_connection() {
    // Arrange
    let (hub_tx, task) = arrange().await;
    ping(&hub_tx).await;

    let user = UserId::default();

    let (h1, mut rx1) = join(&hub_tx, user).await;
    let (h2, mut rx2) = join(&hub_tx, user).await;

    let id1: ConnectionId = h1.id();
    let id2: ConnectionId = h2.id();
    let msg1 = ServerToClient::MessageNotYetKnown;
    let msg2 = ServerToClient::User(UserAction::YouAre(user));

    // Act
    hub_tx
        .send(WsHubMsg::Send {
            user_id: user,
            con_id: id1,
            msg: msg1.clone(),
        })
        .await
        .unwrap();

    // sanity: sending to the other id should reach rx2
    hub_tx
        .send(WsHubMsg::Send {
            user_id: user,
            con_id: id2,
            msg: msg2.clone(),
        })
        .await
        .unwrap();

    let received1 = recv_unbounded(&mut rx1).await;
    let received2 = recv_unbounded(&mut rx2).await;

    // Assert
    assert!(matches!(received1, Some(ServerToClient::MessageNotYetKnown)));
    assert!(matches!(received2, Some(ServerToClient::User(UserAction::YouAre(_)))));
    ping(&hub_tx).await;
    assert_empty_unbounded(&mut rx1);
    assert_empty_unbounded(&mut rx2);

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}

#[tokio::test]
async fn dropping_handle_leaves_and_closes_receiver() {
    // Arrange
    let (hub_tx, task) = arrange().await;
    ping(&hub_tx).await;

    let user = UserId::default();

    // Act
    let (handle, mut rx) = join(&hub_tx, user).await;
    drop(handle); // triggers Leave via Drop

    // Assert
    // after Leave, the hub drops the sender, so the receiver should close
    let got = recv_unbounded(&mut rx).await;
    assert!(got.is_none());

    drop(hub_tx);
    task.abort();
    let _ = task.await; // will be cancelled
}
