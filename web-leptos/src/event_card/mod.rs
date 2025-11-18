use crate::app_state::AppStateReader;
use crate::components::buttons::ServerButton;
use leptos::prelude::*;
use peer_practice_shared::level::Level;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::post::PostId;
use peer_practice_shared::user::UserId;
use pulldown_cmark::{Options, Parser, html};
use std::collections::HashSet;
use std::sync::Arc;

pub mod editable;
pub mod readonly;

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
            state.send(ClientToServer::Leave(props.id));
        } else {
            state.send(ClientToServer::Join(props.id));
        }
    };

    view! {
        <div class="cluster" style="--cluster-justify: space-between; margin-top: 1rem;">
            <div class="cluster" style="--cluster-gap: .75rem; --cluster-justify: flex-start;">
                <span style="min-width: 3rem; text-align: left; opacity: .8;">"Joining"</span>

                <ServerButton
                    class=Signal::derive(move || { "btn".to_string() })
                    data_theme=Arc::new(move || { if partaking() { "success" } else { "primary" } })
                    on_click=Callback::new(move |_| toggle_join())
                >
                    {move || if partaking() { "Joined".to_string() } else { "Join".to_string() }}
                </ServerButton>

                <span style="display: inline-flex; align-items: center; gap: .35rem; opacity: .9;">
                    "👥 " {move || count}
                </span>
            </div>
            <em style="opacity: .8;">{"by "} {props.author.to_string()}</em>
        </div>
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
