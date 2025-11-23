use crate::input::config::current::Config;
use peer_practice_server_services::{email, pending_logins, posts, storage, users, ws_hub};
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
        let storage = storage::spawn_storage_actor(config.server.data_dir.clone());
        let ws_hub = ws_hub::spawn_ws_hub();
        let pending_logins = pending_logins::spawn_pending_logins_actor();
        let users = users::spawn_users_actor(storage.clone(), ws_hub.clone());
        let email = email::spawn_email_actor(
            config
                .email
                .clone()
                .try_into()
                .expect("Invalid email config."),
        );
        let posts = posts::spawn_posts_actor(storage.clone(), ws_hub.clone());

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
