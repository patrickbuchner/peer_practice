use futures_channel::mpsc::UnboundedSender;
use futures_util::SinkExt;
use leptos::prelude::{Get, GetUntracked, ReadSignal, Update, WriteSignal, signal};
use leptos::task::spawn_local;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::post::{Post, PostId};
use peer_practice_shared::user::UserId;
use peer_practice_shared::user::display_user::UserDisplay;
use std::collections::HashMap;

pub fn initialize_app_state() -> (AppStateReader, AppStateWriter) {
    let (tx_read, tx_write) = signal(None);
    let (user_id_read, user_id_write) = signal(None);
    let (posts_read, posts_write) = signal(HashMap::new());
    let (users_read, users_write) = signal(HashMap::new());
    let (pending_route_read, pending_route_write) = signal(None);
    (
        AppStateReader {
            tx: tx_read,
            user_id: user_id_read,
            posts: posts_read,
            users: users_read,
            pending_route: pending_route_read,
        },
        AppStateWriter {
            tx: tx_write,
            user_id: user_id_write,
            posts: posts_write,
            users: users_write,
            pending_route: pending_route_write,
        },
    )
}
#[derive(Copy, Clone)]
pub struct AppStateWriter {
    tx: WriteSignal<Option<UnboundedSender<ClientToServer>>>,
    pub user_id: WriteSignal<Option<UserId>>,
    pub posts: WriteSignal<HashMap<PostId, Post>>,
    pub users: WriteSignal<HashMap<UserId, UserDisplay>>,
    pub pending_route: WriteSignal<Option<String>>,
}
impl AppStateWriter {
    pub(crate) fn set_tx(&self, tx: Option<UnboundedSender<ClientToServer>>) {
        self.tx.update(|s| *s = tx);
    }
}

#[derive(Copy, Clone)]
pub struct AppStateReader {
    tx: ReadSignal<Option<UnboundedSender<ClientToServer>>>,
    pub user_id: ReadSignal<Option<UserId>>,
    pub posts: ReadSignal<HashMap<PostId, Post>>,
    pub users: ReadSignal<HashMap<UserId, UserDisplay>>,
    pub pending_route: ReadSignal<Option<String>>,
}

impl AppStateReader {
    pub(crate) fn connected_to_server(&self) -> bool {
        self.tx.get().is_some()
    }
    pub(crate) fn connected_to_server_untracked(&self) -> bool {
        self.tx.get_untracked().is_some()
    }
    pub fn send(&self, msg: ClientToServer) {
        match self.tx.get_untracked().clone() {
            None => {}
            Some(mut tx) => spawn_local(async move {
                let _ = tx.send(msg).await;
            }),
        }
    }
}
