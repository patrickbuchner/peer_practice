use crate::app_state::{AppStateReader, AppStateWriter};
use crate::host;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures_util::{SinkExt, StreamExt};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use peer_practice_shared::messages::{ClientToServer, ServerToClient};
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
            _ = tx_get.send(ClientToServer::GetPosts).await;
        });
    }));
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onmessage = Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |e: MessageEvent| {
        if let Some(txt) = e.data().as_string() {
            match serde_json::from_str::<ServerToClient>(&txt) {
                Ok(msg) => handle_websocket_messages(write_state, state, msg),
                Err(err) => log!("Failed to deserialize ServerToClient: {}", err),
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
            if let Ok(text) = serde_json::to_string(&msg)
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
        ServerToClient::User(id, user) => {
            state_writer.users.update(|s| {
                s.insert(id, user);
            });
        }
        ServerToClient::Post(id, post) => {
            state_writer.users.update(|s| {
                if !s.contains_key(&post.owner) {
                    state.send(ClientToServer::GetUser(post.owner));
                }
            });
            state_writer.posts.write().insert(id, post);
        }
        ServerToClient::YouAre(id) => {
            state_writer.user_id.set(Some(id));
            state.send(ClientToServer::GetUser(id));
        }
        ServerToClient::RemovedPost(id) => _ = state_writer.posts.write().remove(&id),
    }
}
