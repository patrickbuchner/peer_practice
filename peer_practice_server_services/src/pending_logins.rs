use chrono::{DateTime, Duration, Utc};
use peer_practice_messages::current::email::Email;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

pub enum PendingLoginsMsg {
    GetByAddress {
        address: Email,
        respond_to: oneshot::Sender<Option<u32>>,
    },
    Upsert {
        address: Email,
        code: u32,
    },
    Remove {
        address: Email,
    },
}

pub fn spawn_pending_logins_actor() -> mpsc::Sender<PendingLoginsMsg> {
    let (tx, mut rx) = mpsc::channel::<PendingLoginsMsg>(64);
    let mut state: HashMap<Email, (u32, DateTime<Utc>)> = HashMap::new();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                PendingLoginsMsg::GetByAddress {
                    address,
                    respond_to,
                } => {
                    let now = Utc::now();
                    let val = if let Some((code, set_at)) = state.get(&address) {
                        if *set_at + Duration::minutes(15) > now {
                            Some(*code)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if state.contains_key(&address) && val.is_none() {
                        state.remove(&address);
                    }
                    let _ = respond_to.send(val);
                }
                PendingLoginsMsg::Upsert { address, code } => {
                    state.insert(address, (code, Utc::now()));
                }
                PendingLoginsMsg::Remove { address } => {
                    state.remove(&address);
                }
            }
        }
    });

    tx
}
