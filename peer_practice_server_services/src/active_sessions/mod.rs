use crate::clock::ClockRef;
use crate::storage::StorageMsg;
use chrono::TimeZone;
use peer_practice_messages::current::sessions::{SessionId, SessionInformation};
use peer_practice_messages::current::user::UserId;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ActiveSessionsMsg {
    RemoveObsoleteJwts,
    ValidateJwt(String, oneshot::Sender<Option<String>>),
    InvalidateJwt(String),
    CreateClient(UserId, oneshot::Sender<SessionId>),
    GetSessions(UserId, oneshot::Sender<Vec<SessionInformation>>),
    UpdateSession(UserId, SessionInformation),
    LogOut(UserId, SessionId),
    LogOutAll(UserId),
    Ping(oneshot::Sender<()>),
}

pub async fn handle_active_sessions(
    jwt_expiry_duration: chrono::Duration,
    clock: ClockRef,
    storage: Sender<StorageMsg>,
    mut rx: Receiver<ActiveSessionsMsg>,
) {
    let mut clients = HashMap::new();
    let mut active_clients = HashMap::new();
    let mut dead_jwts = HashMap::new();

    const INVALIDATION_GRACE_PERIOD: chrono::Duration = chrono::Duration::minutes(5);

    let (tx, rxo) = oneshot::channel::<Value>();
    if storage
        .send(StorageMsg::LoadJson {
            namespace: "active_sessions".to_string(),
            respond_to: tx,
        })
        .await
        .is_ok()
        && let Ok(val) = rxo.await
        && let Ok(snap) = serde_json::from_value::<ActiveSessionsSnapshot>(val)
    {
        apply_snapshot(snap, &mut clients, &mut dead_jwts);
    }

    while let Some(msg) = rx.recv().await {
        match msg {
            ActiveSessionsMsg::ValidateJwt(jwt, tx) => {
                let now = clock.now();
                println!("{:?} {:?}", jwt, now);
                let allowed = match dead_jwts.get(&jwt) {
                    None => true,
                    Some(invalidated_at) => now - *invalidated_at <= INVALIDATION_GRACE_PERIOD,
                };

                if allowed {
                    tx.send(Some(jwt)).unwrap();
                } else {
                    tx.send(None).unwrap();
                }
            }
            ActiveSessionsMsg::InvalidateJwt(jwt) => {
                println!("{:?} {:?}", jwt, clock.now());
                dead_jwts.insert(jwt, clock.now());

                persist(storage.clone(), &clients, &dead_jwts).await;
            }
            ActiveSessionsMsg::CreateClient(user_id, tx) => {
                let id = SessionId::new();
                clients.insert(user_id, id);
                if let Entry::Vacant(e) = active_clients.entry(user_id) {
                    e.insert(vec![id]);
                } else {
                    active_clients.get_mut(&user_id).unwrap().push(id);
                }
                _ = tx.send(id);

                persist(storage.clone(), &clients, &dead_jwts).await;
            }
            ActiveSessionsMsg::RemoveObsoleteJwts => {
                let now = clock.now();
                let expired_jwts = dead_jwts
                    .iter()
                    .filter(|(_, timestamp)| now - **timestamp > jwt_expiry_duration)
                    .map(|(jwt, _)| jwt.clone());

                for jwt in expired_jwts.collect::<Vec<_>>() {
                    dead_jwts.remove(&jwt);
                }

                persist(storage.clone(), &clients, &dead_jwts).await;
            }
            ActiveSessionsMsg::Ping(tx) => {
                _ = tx.send(());
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ActiveSessionsSnapshot {
    // Each is serialized as a pair list to keep it JSON-friendly and stable.
    // dead_jwts stores millis since epoch (UTC) to avoid needing chrono serde features.
    clients: Vec<(UserId, SessionId)>,
    dead_jwts: Vec<(String, i64)>,
}

fn snapshot_from_state(
    clients: &HashMap<UserId, SessionId>,
    dead_jwts: &HashMap<String, chrono::DateTime<chrono::Utc>>,
) -> ActiveSessionsSnapshot {
    ActiveSessionsSnapshot {
        clients: clients.iter().map(|(u, c)| (*u, *c)).collect(),
        dead_jwts: dead_jwts
            .iter()
            .map(|(jwt, ts)| (jwt.clone(), ts.timestamp_millis()))
            .collect(),
    }
}

fn apply_snapshot(
    snap: ActiveSessionsSnapshot,
    clients: &mut HashMap<UserId, SessionId>,
    dead_jwts: &mut HashMap<String, chrono::DateTime<chrono::Utc>>,
) {
    clients.clear();
    dead_jwts.clear();

    clients.extend(snap.clients);

    for (jwt, millis) in snap.dead_jwts {
        if let chrono::LocalResult::Single(dt) = chrono::Utc.timestamp_millis_opt(millis) {
            dead_jwts.insert(jwt, dt);
        }
    }
}

async fn persist(
    storage: Sender<StorageMsg>,
    clients: &HashMap<UserId, SessionId>,
    dead_jwts: &HashMap<String, chrono::DateTime<chrono::Utc>>,
) {
    let snap = snapshot_from_state(clients, dead_jwts);
    let data = serde_json::to_value(snap).unwrap_or(Value::Null);

    let _ = storage
        .send(StorageMsg::SaveJson {
            namespace: "active_sessions".to_string(),
            data: json!(data),
        })
        .await;
}

#[cfg(test)]
mod test;
