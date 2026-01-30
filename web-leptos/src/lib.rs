use crate::app_state::{initialize_app_state, AppStateReader};
use crate::event_card::EventCardProps;
use crate::nav_menu::NavMenu;
use crate::components::theme::{AccentStrength, Theme};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{path, NavigateOptions};
use peer_practice_shared::level::Level;
use peer_practice_shared::post::PostId;
use peer_practice_shared::ymd;
use std::collections::HashSet;
use components::styles::button_class::ButtonClass;
use components::styles::color::{StatusColor, SvgStrokeColor};
use components::styles::navbar::NavbarClass;
use components::styles::status::StatusClass;

mod app_state;
mod chat;
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
            <nav class=NavbarClass::Navbar.as_str()>
                <Show
                    when=logged_in
                    fallback=|| {
                        view! {
                            <div class=NavbarClass::SectionCenter.as_str()>
                                <strong class=NavbarClass::Title.as_str()>"Peer Practice"</strong>
                            </div>
                        }
                    }
                >
                    <div class=NavbarClass::Section.as_str()>
                        <NavMenu />
                    </div>
                    <div class=NavbarClass::SectionCenter.as_str()>
                        <strong class=NavbarClass::TitleAccent.as_str()>
                            {active_user_label}
                        </strong>
                    </div>
                    <div class=NavbarClass::SectionEnd.as_str()>
                        <div class=NavbarClass::IconBar.as_str()>
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
                                        path=path!("/chat/:chat_id")
                                        view=move || view! { <chat::ChatRoute state /> }
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
            class=ButtonClass::Fab.as_str()
            data-theme=Theme::Accent.as_str()
            data-accent-strength=AccentStrength::Strong.as_str()
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
            <span class=ButtonClass::Icon.as_str()>
                {move || {
                    if read_new_post.get().is_some() { "-".to_string() } else { "+".to_string() }
                }}
            </span>
        </button>
    }
}

#[component]
fn ConnectionStatus(state: AppStateReader) -> impl IntoView {
    let color =
        move || StatusColor::from_connected(state.connected_to_server()).as_str();
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
            class=StatusClass::Indicator.as_str()
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
                    stroke=SvgStrokeColor::StatusOutline.as_str()
                    stroke-width="1"
                    fill=move || color().to_string()
                />
            </svg>
            <Show when=move || show_toast.get()>
                <div
                    role="status"
                    class=StatusClass::Toast.as_str()
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
