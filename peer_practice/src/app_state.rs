use crate::input::config::current::Config;
use peer_practice_server_services::chat::handle_chats;
use peer_practice_server_services::clock;
use peer_practice_server_services::email::handle_email_actions;
use peer_practice_server_services::pending_logins::handle_pending_logins;
use peer_practice_server_services::posts::handle_posts;
use peer_practice_server_services::storage::handle_storage_operations;
use peer_practice_server_services::users::handle_user_actions;
use peer_practice_server_services::ws_hub::handle_ws_hub_actions;
use peer_practice_server_services::{chat, email, pending_logins, posts, users, ws_hub};
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
    pub chat: Sender<chat::ChatMsg>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let storage_config = config.server.data_dir.clone();
        let (ws_hub, rx) = mpsc::channel::<ws_hub::WsHubMsg>(128);
        tokio::spawn(handle_ws_hub_actions(rx, ws_hub.clone()));

        let clock = clock::system_clock();
        let storage = spawn(128, |rx| handle_storage_operations(storage_config, rx));
        let email_config = config.email.clone().try_into().expect("Invalid email");
        let email = spawn(64, |rx| handle_email_actions(email_config, rx));
        let pending_logins = spawn(64, |rx| handle_pending_logins(clock.clone(), rx));
        let users = spawn(64, |rx| {
            handle_user_actions(storage.clone(), ws_hub.clone(), rx)
        });
        let chat = spawn(100, |rx| handle_chats(storage.clone(), ws_hub.clone(), clock.clone(), rx));
        let posts = spawn(100, |rx| handle_posts(storage.clone(), ws_hub.clone(), chat.clone(), rx));

        Self {
            jwt_secret: config.server.jwt_secret.clone(),
            pending_logins,
            users,
            email,
            posts,
            ws_hub,
            chat,
        }
    }
}

/// Helper function to streamline actor spawning.
/// It infers the message type M from the closure's receiver.
fn spawn<M, F, Fut>(buffer: usize, f: F) -> Sender<M>
where
    M: Send + 'static,
    F: FnOnce(mpsc::Receiver<M>) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::channel(buffer);
    tokio::spawn(f(rx));
    tx
}
