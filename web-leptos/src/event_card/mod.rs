use crate::app_state::AppStateReader;
use crate::components::buttons::ServerButton;
use crate::components::styles::color::ShadowColor;
use crate::components::theme::{IntentTheme, Theme};
use chrono::NaiveDate;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use peer_practice_shared::level::Level;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::messages::client_to_server::ChatAction;
use peer_practice_shared::post::PostId;
use peer_practice_shared::user::UserId;
use peer_practice_shared::ymd;
use pulldown_cmark::{Options, Parser, html};
use std::collections::HashSet;
use std::sync::Arc;

pub mod editable;
pub mod readonly;

use crate::components::styles::button_class::ButtonClass;
use crate::components::styles::event_card::EventCardClass;
use peer_practice_shared::messages::client_to_server::PostAction;

#[derive(Clone, PartialEq)]
pub struct EventCardProps {
    pub id: PostId,
    pub title: String,
    pub date: String,
    pub level: Level,
    pub ideas: String,
    pub partaking: HashSet<UserId>,
    pub author: String,
}

fn event_card_footer(props: EventCardProps, state: AppStateReader) -> impl IntoView + use<> {
    let post_id = props.id;
    let partaking = move || match state.user_id.get() {
        None => false,
        Some(id) => {
            if post_id == PostId::NULL {
                false
            } else {
                state
                    .posts
                    .get()
                    .get(&post_id)
                    .unwrap()
                    .partaking_users
                    .contains(&id)
            }
        }
    };
    let count = move || {
        if post_id == PostId::NULL {
            0
        } else {
            state
                .posts
                .get()
                .get(&post_id)
                .unwrap()
                .partaking_users
                .len()
        }
    };

    let toggle_join = move || {
        if partaking() {
            state.send(ClientToServer::Post(PostAction::Leave(props.id)));
        } else {
            state.send(ClientToServer::Post(PostAction::Join(props.id)));
        }
    };

    view! {
        <div class=EventCardClass::Footer.as_str()>
            <div class=EventCardClass::FooterCluster.as_str()>
                <span class=EventCardClass::Label.as_str()>"Joining"</span>

                <ServerButton
                    class=Signal::derive(move || { ButtonClass::Base.as_str().to_string() })
                    data_theme=Arc::new(move || {
                        if partaking() {
                            Theme::Intent(IntentTheme::Success)
                        } else {
                            Theme::Intent(IntentTheme::Primary)
                        }
                    })
                    on_click=Callback::new(move |_| toggle_join())
                >
                    {move || if partaking() { "Joined".to_string() } else { "Join".to_string() }}
                </ServerButton>

                <span class=EventCardClass::Count.as_str()>"👥 " {move || count}</span>

                <ChatButton post_id state />
            </div>
            <em class=EventCardClass::Author.as_str()>{"by "} {props.author.to_string()}</em>
        </div>
    }
}

#[component]
fn ChatButton(post_id: PostId, #[prop(into)] state: AppStateReader) -> impl IntoView {
    let (waiting_for_chat, set_waiting_for_chat) = signal(false);
    let navigate = use_navigate();
    let navigate_effect = navigate.clone();
    let location = use_location();
    let is_disabled = Signal::derive(move || post_id == PostId::NULL);
    let is_on_chat = Signal::derive(move || location.pathname.get().starts_with("/chat/"));

    Effect::new(move |_| {
        if waiting_for_chat.get() {
            if let Some(chat_id) = state.post_chats.get().get(&post_id).cloned() {
                let path = format!("/chat/{}", chat_id.get_id());
                navigate_effect(&path, Default::default());
                set_waiting_for_chat.set(false);
            }
        }
    });

    view! {
        <ServerButton
            class=Signal::derive(move || { ButtonClass::Base.as_str().to_string() })
            disabled=is_disabled
            data_theme=Arc::new(|| Theme::Intent(IntentTheme::Primary))
            on_click=Callback::new(move |_| {
                if is_on_chat.get_untracked() {
                    navigate("/", Default::default());
                    set_waiting_for_chat.set(false);
                    return;
                }
                if post_id != PostId::NULL {
                    state.send(ClientToServer::Chat(ChatAction::GetChatFor(post_id)));
                    set_waiting_for_chat.set(true);
                }
            })
        >
            {move || if is_on_chat.get() { "Back to overview" } else { "Chat" }}
        </ServerButton>
    }
}

fn markdown_to_safe_html(src: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    opts.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(src, opts);

    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);

    ammonia::Builder::default().clean(&html_buf).to_string()
}

pub fn shadow_color_for_date(date_str: &str) -> ShadowColor {
    let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") else {
        return ShadowColor::Base;
    };
    let today = chrono::Local::now().date_naive();
    if date < today {
        return ShadowColor::Base;
    }

    let options = ymd::create_date_options();
    let Some(index) = options.iter().position(|opt| opt == date_str) else {
        return ShadowColor::Base;
    };

    match index {
        0 => ShadowColor::Green,
        1 => ShadowColor::Teal,
        2 => ShadowColor::Sky,
        3 => ShadowColor::Mauve,
        4 => ShadowColor::Lavender,
        _ => ShadowColor::Base,
    }
}
