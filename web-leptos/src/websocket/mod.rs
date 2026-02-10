use crate::app_state::{AppStateReader, AppStateWriter};
use crate::host;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures_util::{SinkExt, StreamExt};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use peer_practice_shared::Envelope;
use peer_practice_shared::messages::server_to_client::{
    ChatAction, PostAction, SessionAction, UserAction,
};
use peer_practice_shared::messages::{ClientToServer, ServerToClient, client_to_server};
use std::cell::Cell;
use std::rc::Rc;
use web_sys::wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

pub fn attempt_connect(
    write_state: AppStateWriter,
    state: AppStateReader,
    first_ws_attempt_completed: WriteSignal<bool>,
) {
    connect(write_state, state, first_ws_attempt_completed, 0);
}

fn backoff_ms(count: u8) -> i32 {
    let factor = 1i32 << count.min(7);
    let base_ms = 250;
    let max_ms = 30_000;
    (factor * base_ms).min(max_ms)
}

fn connect(
    write_state: AppStateWriter,
    state: AppStateReader,
    first_ws_attempt_completed: WriteSignal<bool>,
    count: u8,
) {
    if state.connected_to_server_untracked() {
        return;
    }

    let timeout = if count == 0 && window().navigator().on_line() {
        0
    } else {
        backoff_ms(count)
    };

    if timeout > 0 {
        let cb =
            Closure::once(move || connect(write_state, state, first_ws_attempt_completed, count));
        let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            timeout,
        );
        cb.forget();
        return;
    }

    let protocol = window()
        .location()
        .protocol()
        .unwrap_or_else(|_| "http:".into());
    let ws_scheme = if protocol == "https:" { "wss" } else { "ws" };
    let url = format!("{ws_scheme}://{}/v1/ws", host());

    let ws = match WebSocket::new(&url) {
        Ok(ws) => ws,
        Err(_) => {
            first_ws_attempt_completed.set(true);

            connect(write_state, state, first_ws_attempt_completed, count + 1);
            return;
        }
    };

    let (tx, mut rx): (
        UnboundedSender<ClientToServer>,
        UnboundedReceiver<ClientToServer>,
    ) = unbounded();

    let connected = Rc::new(Cell::new(false));
    let connected_onopen = connected.clone();
    let onopen = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        connected_onopen.set(true);
        let mut tx_get = tx.clone();
        write_state.set_tx(Some(tx.clone()));
        if count == 0 {
            first_ws_attempt_completed.set(true);
        }
        spawn_local(async move {
            _ = tx_get.send(ClientToServer::Hello).await;
        });
    }));
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            log!("Received message: {}", txt);
            match serde_json::from_str::<Envelope<ServerToClient>>(&txt) {
                Ok(envelope) => handle_websocket_messages(write_state, state, envelope.data),
                Err(err) => log!("Failed to deserialize Envelope<ServerToClient>: {}", err),
            }
        }
    }));
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onclose = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        log!("WebSocket closed start");
        write_state.set_tx(None);
        first_ws_attempt_completed.set(true);
        log!("WebSocket closed raise event");

        let next_count = count + 1;
        connect(write_state, state, first_ws_attempt_completed, next_count);
        log!("WebSocket closed done");
    }));
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();

    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            let envelope = peer_practice_shared::create_envelope(msg);
            if let Ok(text) = serde_json::to_string(&envelope)
                && let Err(e) = ws.clone().send_with_str(&text)
            {
                log!(
                    "Failed to send message: {}",
                    e.as_string().unwrap_or_default()
                );
            }
        }
    });
}

fn handle_websocket_messages(
    state_writer: AppStateWriter,
    state: AppStateReader,
    msg: ServerToClient,
) {
    match msg {
        ServerToClient::User(user_action) => match user_action {
            UserAction::User(id, user) => {
                state_writer.users.update(|s| {
                    s.insert(id, user);
                });
            }
            UserAction::YouAre(id) => {
                state_writer.user_id.set(Some(id));
                state.send(ClientToServer::User(client_to_server::UserAction::Get(id)));
                state.send(ClientToServer::Post(client_to_server::PostAction::GetPosts));
                state.send(ClientToServer::Session(client_to_server::SessionAction::GetSessions));
                state.send(ClientToServer::Session(client_to_server::SessionAction::GetThisSession));
            }
        },
        ServerToClient::Post(post_action) => match post_action {
            PostAction::Post(id, post) => {
                state_writer.users.update(|s| {
                    if !s.contains_key(&post.owner) {
                        state.send(ClientToServer::User(client_to_server::UserAction::Get(
                            post.owner,
                        )));
                    }
                });
                state_writer.posts.write().insert(id, post);
            }
            PostAction::RemovedPost(id) => _ = state_writer.posts.write().remove(&id),
        },
        ServerToClient::Chat(chat_action) => match chat_action {
            ChatAction::ChatDoesNotExistForPost(_) => {}
            ChatAction::ChatDoesNotExist(_) => {}
            ChatAction::Chat(chat_id, post_id, messages) => {
                state_writer.chats.write().insert(chat_id, messages);
                state_writer.chat_posts.write().insert(chat_id, post_id);
                state_writer.post_chats.write().insert(post_id, chat_id);
            }
            ChatAction::MessageSent(message) => {
                state_writer.chats.update(|chats| {
                    chats.entry(message.chat_id).or_default().push(message);
                });
            }
        },
        ServerToClient::Session(session_action) => match session_action {
            SessionAction::CurrentSession(sid) => {
                state_writer.sessions.current.set(Some(sid));
            }
            SessionAction::Sessions(sessions) => state_writer.sessions.sessions.update(|s| {
                s.clear();
                for session in sessions {
                    s.insert(session.session_id, session);
                }
            }),
            SessionAction::SessionInformationChanged(info) => {
                state_writer.sessions.sessions.update(|s| {
                    if s.contains_key(&info.session_id)
                        && let Some(s) = s.get_mut(&info.session_id)
                    {
                        s.description = info.description.clone();
                    }
                })
            }
        },
        _ => log!("Received unhandled message type"),
    }
}
