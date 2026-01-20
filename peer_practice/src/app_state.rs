use crate::input::config::current::Config;
use peer_practice_server_services::email::{EmailConfiguration, EmailMsg, handle_email_actions};
use peer_practice_server_services::pending_logins::{PendingLoginsMsg, handle_pending_logins};
use peer_practice_server_services::posts::{PostsMsg, handle_posts};
use peer_practice_server_services::storage::{StorageMsg, handle_storage_operations};
use peer_practice_server_services::users::{UsersMsg, handle_user_operations};
use peer_practice_server_services::ws_hub::{WsHubMsg, handle_ws_hub_actions};
use peer_practice_server_services::{email, pending_logins, posts, storage, users, ws_hub};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub pending_logins: Sender<pending_logins::PendingLoginsMsg>,
    pub users: Sender<users::UsersMsg>,
    pub email: Sender<email::EmailMsg>,
    pub posts: Sender<posts::PostsMsg>,
    pub ws_hub: Sender<ws_hub::WsHubMsg>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let storage = spawn_storage_actor(config.server.data_dir.clone());
        let ws_hub = spawn_ws_hub();
        let pending_logins = spawn_pending_logins_actor();
        let users = spawn_users_actor(storage.clone(), ws_hub.clone());
        let email = spawn_email_actor(
            config
                .email
                .clone()
                .try_into()
                .expect("Invalid email config."),
        );
        let posts = spawn_posts_actor(storage.clone(), ws_hub.clone());

        Self {
            jwt_secret: config.server.jwt_secret.clone(),
            pending_logins,
            users,
            email,
            posts,
            ws_hub,
        }
    }
}

pub fn spawn_posts_actor(
    storage: Sender<StorageMsg>,
    ws_hub: Sender<WsHubMsg>,
) -> Sender<PostsMsg> {
    let (tx, rx) = mpsc::channel::<PostsMsg>(100);
    tokio::spawn(handle_posts(storage, ws_hub, rx));
    tx
}

pub fn spawn_pending_logins_actor() -> mpsc::Sender<PendingLoginsMsg> {
    let (tx, rx) = mpsc::channel::<PendingLoginsMsg>(64);
    tokio::spawn(handle_pending_logins(rx));
    tx
}
pub fn spawn_storage_actor(work_dir: PathBuf) -> mpsc::Sender<StorageMsg> {
    let (tx, rx) = mpsc::channel::<StorageMsg>(128);
    tokio::spawn(handle_storage_operations(work_dir, rx));
    tx
}
pub fn spawn_users_actor(
    storage: Sender<StorageMsg>,
    ws_hub: Sender<WsHubMsg>,
) -> Sender<UsersMsg> {
    let (tx, rx) = mpsc::channel::<UsersMsg>(64);
    tokio::spawn(handle_user_operations(storage, ws_hub, rx));
    tx
}

pub fn spawn_ws_hub() -> mpsc::Sender<WsHubMsg> {
    let (tx, rx) = mpsc::channel::<WsHubMsg>(128);
    tokio::spawn(handle_ws_hub_actions(rx, tx.clone()));
    tx
}

pub fn spawn_email_actor(config: EmailConfiguration) -> mpsc::Sender<EmailMsg> {
    let (tx, rx) = mpsc::channel::<EmailMsg>(64);
    tokio::spawn(handle_email_actions(config, rx));
    tx
}
