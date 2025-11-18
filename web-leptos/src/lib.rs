use crate::app_state::{AppStateReader, initialize_app_state};
use crate::event_card::EventCardProps;
use crate::nav_menu::NavMenu;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{NavigateOptions, path};
use peer_practice_shared::level::Level;
use peer_practice_shared::post::PostId;
use peer_practice_shared::ymd;
use std::collections::HashSet;

mod app_state;
mod components;
pub mod event_card;
pub mod home;
mod login;
mod settings;
mod websocket;

#[component]
pub fn App() -> impl IntoView {
    let (state, write_state) = initialize_app_state();
    provide_context(state);
    provide_context(write_state);

    let loc = window().location();
    log!("Current location: {}", loc.pathname().unwrap_or_default());
    write_state
        .pending_route
        .set(Some(loc.pathname().unwrap_or_default()));

    let (first_ws_attempt_complete_read, first_ws_attempt_complete_write) = signal(false);
    let (read_new_post, write_new_post) = signal::<Option<EventCardProps>>(None);
    provide_context(read_new_post);
    provide_context(write_new_post);

    Effect::new(move |_| {
        log!("Redirecting on ws state");
        log!(
            "First ws attempt complete: {}",
            first_ws_attempt_complete_read.get()
        );
        log!("Connected {}", state.connected_to_server());

        let navigate = leptos_router::hooks::use_navigate();
        if first_ws_attempt_complete_read.get() && !state.connected_to_server() {
            navigate("/login", NavigateOptions::default());
            return;
        }

        if first_ws_attempt_complete_read.get() && state.connected_to_server() {
            let path = state.pending_route.get();
            let path = if let Some(path) = path
                && !path.starts_with("/login")
            {
                path
            } else {
                "/".into_owned()
            };
            log!("Redirecting to {}", path);
            navigate(&path, Default::default());
            *write_state.pending_route.write_untracked() = None;
        }
    });

    websocket::attempt_connect(write_state, state, first_ws_attempt_complete_write);

    let connected = move || state.connected_to_server();
    let logged_in = move || state.user_id.get().is_some();

    let active_user_label = move || {
        if let Some(uid) = state.user_id.get() {
            if let Some(user) = state.users.get().get(&uid) {
                user.display_name.clone().unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };
    view! {
        <Router>
            <nav
                class="navbar"
                style="display: flex; align-items: center; padding: 0.5rem 0.75rem; gap: 0;"
            >
                <Show
                    when=logged_in
                    fallback=|| {
                        view! {
                            <div style="flex: 1; display:flex; align-items:center; justify-content:center;">
                                <strong style="font-size: 1.375rem;">"Peer Practice"</strong>
                            </div>
                        }
                    }
                >
                    <div style="flex: 1; display:flex; align-items:center;">
                        <NavMenu />
                    </div>
                    <div style="flex: 1; display:flex; align-items:center; justify-content:center;">
                        <strong style="font-size: 1.375rem; color: var(--teal);">
                            {active_user_label}
                        </strong>
                    </div>
                    <div style="flex: 1; display:flex; justify-content:flex-end;">
                        <div
                            class="nav-icon-bar"
                            style="display: flex; align-items: center; gap: 0.5rem;"
                        >
                            <CreateNewPost state read_new_post write_new_post />
                            <ConnectionStatus state />
                        </div>
                    </div>
                </Show>
            </nav>
            <main>
                <Show
                    when=move || first_ws_attempt_complete_read.get()
                    fallback=|| view! { <p>"Loading..."</p> }
                >
                    {move || {
                        if !connected() {
                            view! {
                                <Routes fallback=move || {
                                    view! {
                                        <login::LoginRoute
                                            state
                                            write_state
                                            first_attempt_completed=first_ws_attempt_complete_write
                                        />
                                    }
                                }>
                                    <Route
                                        path=path!("/login")
                                        view=move || {
                                            view! {
                                                <login::LoginRoute
                                                    state
                                                    write_state
                                                    first_attempt_completed=first_ws_attempt_complete_write
                                                />
                                            }
                                        }
                                    />
                                </Routes>
                            }
                                .into_any()
                        } else {
                            view! {
                                <Routes fallback=move || view! { <home::Home state /> }>
                                    <Route
                                        path=path!("/")
                                        view=move || view! { <home::Home state /> }
                                    />
                                    <Route
                                        path=path!("/settings")
                                        view=move || view! { <settings::Settings state /> }
                                    />
                                </Routes>
                            }
                                .into_any()
                        }
                    }}
                </Show>
            </main>
        </Router>
    }
}
mod nav_menu;

#[component]
fn CreateNewPost(
    state: AppStateReader,
    read_new_post: ReadSignal<Option<EventCardProps>>,
    write_new_post: WriteSignal<Option<EventCardProps>>,
) -> impl IntoView {
    view! {
        <button
            aria-label="Add post"
            title="Add post"
            style="
            width: 36px;
            height: 36px;
            border-radius: 9999px;
            display: flex;
            align-items: center;
            justify-content: center;
            background: var(--mauve, #14b8a6);
            color: white;
            border: none;
            box-shadow: 0 4px 12px rgba(0,0,0,0.2);
            cursor: pointer;
            "
            on:click=move |_| {
                let current_user = state.user_id.get();
                let author_name = current_user
                    .and_then(|uid| {
                        state.users.get().get(&uid).and_then(|u| u.display_name.clone())
                    })
                    .unwrap_or_else(|| "-".to_string());
                let draft = EventCardProps {
                    id: PostId::NULL,
                    title: String::new(),
                    date: ymd::create_date_options().first().unwrap().clone(),
                    level: Level::Beginner1,
                    ideas: String::new(),
                    partaking: HashSet::new(),
                    author: author_name,
                };
                if read_new_post.get().is_some() {
                    write_new_post.set(None);
                } else {
                    window().scroll_to_with_x_and_y(0.0, 0.0);
                    write_new_post.set(Some(draft));
                }
            }
        >
            <span style="font-size: 22px; line-height: 1; margin-top: -1px;">
                {move || {
                    if read_new_post.get().is_some() { "-".to_string() } else { "+".to_string() }
                }}
            </span>
        </button>
    }
}

#[component]
fn ConnectionStatus(state: AppStateReader) -> impl IntoView {
    let color = move || {
        if state.connected_to_server() {
            "var(--success-color)"
        } else {
            "var(--danger-color)"
        }
    };
    let status_text = move || {
        if state.connected_to_server() {
            "Connected to server"
        } else {
            "Disconnected from server"
        }
    };
    let (show_toast, set_show_toast) = signal(false);
    view! {
        <div
            class="connection-status"
            style="display:flex; align-items:center; gap:0.35rem; position:relative;"
            on:mouseenter=move |_| set_show_toast.set(true)
            on:mouseleave=move |_| set_show_toast.set(false)
        >
            <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                aria-label="Connection status"
                role="img"
            >
                <circle
                    cx="12"
                    cy="12"
                    r="8"
                    stroke="#111827"
                    stroke-width="1"
                    fill=move || color().to_string()
                />
            </svg>
            <Show when=move || show_toast.get()>
                <div
                    role="status"
                    style="
                    position: fixed;
                    top: calc(var(--navbar-height, 48px) + 6px);
                    right: 0.75rem;
                    background: #111827;
                    color: white;
                    padding: 0.4rem 0.6rem;
                    border-radius: 0.375rem;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                    font-size: 0.875rem;
                    z-index: 2000;
                    white-space: nowrap;
                    pointer-events: none;
                    "
                >
                    {status_text}
                </div>
            </Show>
        </div>
    }
}

pub fn host() -> String {
    let window = window();
    let location = window.location();
    location.hostname().expect("should have a URL")
}
