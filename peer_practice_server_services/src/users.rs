use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use crate::storage::StorageMsg;
use crate::ws_hub::WsHubMsg;
use peer_practice_messages::current::email::Email;
use peer_practice_messages::current::messages::ServerToClient;
use peer_practice_messages::current::user::{User, UserId};
use tokio::sync::mpsc::Sender;
use tracing::{error, info};

pub enum UsersMsg {
    GetByEmail {
        email: Email,
        respond_to: oneshot::Sender<Option<UserId>>,
    },
    GetById {
        id: UserId,
        respond_to: oneshot::Sender<Option<User>>,
    },
    Update {
        id: UserId,
        user: User,
    },
    Remove {
        id: UserId,
    },
}

pub fn spawn_users_actor(
    storage: Sender<StorageMsg>,
    ws_hub: Sender<WsHubMsg>,
) -> Sender<UsersMsg> {
    let (tx, mut rx) = mpsc::channel::<UsersMsg>(64);

    let mut id_to_user: HashMap<UserId, User> = HashMap::new();
    let mut email_to_id: HashMap<Email, UserId> = HashMap::new();

    tokio::spawn(async move {
        setup(&storage, &mut id_to_user, &mut email_to_id).await;

        while let Some(msg) = rx.recv().await {
            match msg {
                UsersMsg::GetByEmail { email, respond_to } => {
                    let val = if let Some(existing) = email_to_id.get(&email).copied() {
                        Some(existing)
                    } else {
                        let mut id = UserId::new();
                        while id_to_user.contains_key(&id) || email_to_id.values().any(|v| *v == id)
                        {
                            id = UserId::new();
                        }

                        email_to_id.insert(email.clone(), id);
                        id_to_user.insert(
                            id,
                            User {
                                id,
                                email,
                                display_name: None,
                            },
                        );
                        let _ = storage
                            .send(StorageMsg::SaveUsers(id_to_user.clone()))
                            .await;
                        Some(id)
                    };
                    let _ = respond_to.send(val);
                }
                UsersMsg::Update { id, user } => {
                    if let Some(existing) = id_to_user.get(&id)
                        && existing.email != user.email
                    {
                        email_to_id.remove(&existing.email);
                    }
                    id_to_user.insert(id, user.clone());
                    email_to_id.insert(user.email.clone(), id);

                    let _ = storage
                        .send(StorageMsg::SaveUsers(id_to_user.clone()))
                        .await;
                    let _ = ws_hub
                        .send(WsHubMsg::BroadcastAll(ServerToClient::User(
                            id,
                            user.into(),
                        )))
                        .await;
                }
                UsersMsg::Remove { id } => {
                    if let Some(removed) = id_to_user.remove(&id) {
                        email_to_id.remove(&removed.email);
                    }

                    let _ = storage
                        .send(StorageMsg::SaveUsers(id_to_user.clone()))
                        .await;
                }
                UsersMsg::GetById { id, respond_to } => {
                    let val = id_to_user.get(&id).cloned();
                    let _ = respond_to.send(val);
                }
            }
        }
    });

    tx
}

async fn setup(
    storage: &Sender<StorageMsg>,
    id_to_user: &mut HashMap<UserId, User>,
    email_to_id: &mut HashMap<Email, UserId>,
) {
    let (respond_to, recv) = oneshot::channel();
    let _ = storage.send(StorageMsg::RetrieveUsers { respond_to }).await;
    match recv.await {
        Ok(entries) => {
            for (id, user) in entries {
                info!("User setup {:?}", user);
                let user = &user;
                let email = user.email.clone();
                email_to_id.insert(email, id);
                id_to_user.insert(id, user.clone());
            }
        }
        Err(e) => {error!("Failed to retrieve users: {}", e)}
    }

}
