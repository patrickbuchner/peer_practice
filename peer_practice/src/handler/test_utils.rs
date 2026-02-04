use crate::app_state::AppState;
use peer_practice_server_services::{chat, email, pending_logins, posts, users, ws_hub};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

pub struct TestReceivers {
    pub pending_logins: mpsc::Receiver<pending_logins::PendingLoginsMsg>,
    pub users: mpsc::Receiver<users::UsersMsg>,
    pub email: mpsc::Receiver<email::EmailMsg>,
    pub posts: mpsc::Receiver<posts::PostsMsg>,
    pub ws_hub: mpsc::Receiver<ws_hub::WsHubMsg>,
    pub chat: mpsc::Receiver<chat::ChatMsg>,
}

pub fn test_state() -> (AppState, TestReceivers) {
    let (pending_logins, pending_logins_rx) = mpsc::channel(8);
    let (users, users_rx) = mpsc::channel(8);
    let (email, email_rx) = mpsc::channel(8);
    let (posts, posts_rx) = mpsc::channel(8);
    let (ws_hub, ws_hub_rx) = mpsc::channel(8);
    let (chat, chat_rx) = mpsc::channel(8);

    let state = AppState {
        jwt_secret: "test-secret".to_string(),
        pending_logins,
        users,
        email,
        posts,
        ws_hub,
        chat,
    };

    (
        state,
        TestReceivers {
            pending_logins: pending_logins_rx,
            users: users_rx,
            email: email_rx,
            posts: posts_rx,
            ws_hub: ws_hub_rx,
            chat: chat_rx,
        },
    )
}

pub async fn recv_msg<T>(rx: &mut mpsc::Receiver<T>) -> T {
    match rx.recv().await {
        Some(msg) => msg,
        None => panic!("channel closed"),
    }
}

pub fn assert_empty<T>(rx: &mut mpsc::Receiver<T>) {
    match rx.try_recv() {
        Ok(_) => panic!("expected no message"),
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => panic!("channel closed"),
    }
}
