use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::user::UserId;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ConnectionId {
    id: Uuid,
}

pub enum WsHubMsg {
    Join {
        user_id: UserId,
        respond_to: oneshot::Sender<(ConnectionHandle, mpsc::UnboundedReceiver<ServerToClient>)>,
    },
    Leave {
        user_id: UserId,
        connection_id: ConnectionId,
    },
    BroadcastAll(ServerToClient),
    BroadcastUser {
        user_id: UserId,
        msg: ServerToClient,
    },
}

#[derive(Clone)]
pub struct ConnectionHandle {
    hub_tx: mpsc::Sender<WsHubMsg>,
    user_id: UserId,
    connection_id: ConnectionId,
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        let _ = self.hub_tx.try_send(WsHubMsg::Leave {
            user_id: self.user_id,
            connection_id: self.connection_id,
        });
    }
}

pub fn spawn_ws_hub() -> mpsc::Sender<WsHubMsg> {
    let (tx, mut rx) = mpsc::channel::<WsHubMsg>(128);
    let mut groups: HashMap<UserId, HashMap<ConnectionId, mpsc::UnboundedSender<ServerToClient>>> =
        HashMap::new();

    let hub_tx_for_handles = tx.clone();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                WsHubMsg::Join {
                    user_id,
                    respond_to,
                } => {
                    let (conn_tx, conn_rx) = mpsc::unbounded_channel();
                    let conn_id = ConnectionId { id: Uuid::new_v4() };

                    groups.entry(user_id).or_default().insert(conn_id, conn_tx);
                    let handle = ConnectionHandle {
                        hub_tx: hub_tx_for_handles.clone(),
                        user_id,
                        connection_id: conn_id,
                    };

                    let _ = respond_to.send((handle, conn_rx));
                }
                WsHubMsg::Leave {
                    user_id,
                    connection_id,
                } => {
                    let _ = groups.get_mut(&user_id).unwrap().remove(&connection_id);
                }
                WsHubMsg::BroadcastAll(msg) => {
                    for sender in groups.values_mut().flat_map(|con| con.values_mut()) {
                        let _ = sender.send(msg.clone());
                    }
                }

                WsHubMsg::BroadcastUser { user_id, msg } => match groups.get_mut(&user_id) {
                    None => {}
                    Some(cons) => cons.values_mut().for_each(|sender| {
                        let _ = sender.send(msg.clone());
                    }),
                },
            }
        }
    });

    tx
}
