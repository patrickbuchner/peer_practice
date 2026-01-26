use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::user::UserId;
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ConnectionId {
    id: Uuid,
}

#[cfg(test)]
mod test;

impl ConnectionId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[cfg_attr(test, derive(Debug))]
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
    Send {
        user_id: UserId,
        con_id: ConnectionId,
        msg: ServerToClient,
    },
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct ConnectionHandle {
    hub_tx: Sender<WsHubMsg>,
    user_id: UserId,
    connection_id: ConnectionId,
}

impl ConnectionHandle {
    pub fn id(&self) -> ConnectionId {
        self.connection_id
    }
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        let _ = self.hub_tx.try_send(WsHubMsg::Leave {
            user_id: self.user_id,
            connection_id: self.connection_id,
        });
    }
}

pub async fn handle_ws_hub_actions(
    mut rx: Receiver<WsHubMsg>,
    hub_tx_for_handles: Sender<WsHubMsg>,
) {
    let mut groups: HashMap<UserId, HashMap<ConnectionId, mpsc::UnboundedSender<ServerToClient>>> =
        HashMap::new();
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
            WsHubMsg::Send {
                user_id,
                con_id,
                msg,
            } => {
                if let Some(sender) = groups.get_mut(&user_id).and_then(|con| con.get(&con_id)) {
                    let _ = sender.send(msg);
                }
            }
        }
    }
}
