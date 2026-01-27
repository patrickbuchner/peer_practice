use crate::app_state::AppStateReader;
use crate::event_card::{EventCardProps, readonly::EventCardReadonly};
use leptos::prelude::*;
use leptos_router::params::Params;
use leptos_router::hooks::use_params;
use peer_practice_shared::chat::ChatId;
use peer_practice_shared::convert_utc_to_local;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::messages::client_to_server::ChatAction;
use peer_practice_shared::post::PostId;
use peer_practice_shared::user::UserId;
use uuid::Uuid;
use std::collections::HashSet;

#[derive(Params, PartialEq, Clone)]
struct ChatParams {
    chat_id: Option<String>,
}

#[component]
pub fn ChatRoute(#[prop(into)] state: AppStateReader) -> impl IntoView {
    let params = use_params::<ChatParams>();

    let chat_id = Memo::new(move |_| {
        params.with(|params| {
            params
                .as_ref()
                .ok()
                .and_then(|p| p.chat_id.as_ref())
                .and_then(|raw| Uuid::parse_str(raw).ok())
                .map(ChatId::from_id)
        })
    });

    Effect::new(move |_| {
        if let Some(chat_id) = chat_id.get() {
            state.send(ClientToServer::Chat(ChatAction::GetChat(chat_id)));
        }
    });

    let associated_post = move || {
        let chat_id = chat_id.get()?;
        let post_id = state.chat_posts.get().get(&chat_id).cloned()?;
        build_event_card(post_id, &state)
    };

    let messages = move || {
        let chat_id = chat_id.get()?;
        state.chats.get().get(&chat_id).cloned()
    };

    let has_valid_chat_id = move || chat_id.get().is_some();

    view! {
        <div style="display:flex; flex-direction:column; gap: 1.5rem; padding: 1rem;">
            <Show
                when=has_valid_chat_id
                fallback=move || view! {
                    <div class="card" data-theme="weak">
                        <h3 class="card-title">"Invalid chat id"</h3>
                        <p style="opacity: 0.8; margin-top: 0.35rem;">
                            "The chat id in the URL could not be parsed."
                        </p>
                    </div>
                }
            >
                <div class="chat-row">
                    <Show
                        when=move || associated_post().is_some()
                        fallback=move || view! {
                            <div class="card" data-theme="weak">
                                <h3 class="card-title">"Associated post unavailable"</h3>
                                <p style="opacity: 0.8; margin-top: 0.35rem;">
                                    "We could not find a post linked to this chat yet."
                                </p>
                            </div>
                        }
                    >
                        {move || {
                            associated_post()
                                .map(|props| view! { <EventCardReadonly props state /> })
                        }}
                    </Show>
                </div>

                <div class="chat-row">
                    <div class="card" data-theme="weak">
                    <div
                        class="cluster"
                        style="--cluster-justify: space-between; --cluster-gap: .5rem; align-items:center;"
                    >
                        <h3 class="card-title">"Chat"</h3>
                        <span style="opacity: .7;">
                            {move || messages().map(|m| m.len()).unwrap_or(0)} " messages"
                        </span>
                    </div>

                    <div
                        style="
                            display:flex;
                            flex-direction:column;
                            gap: .75rem;
                            margin-top: 1rem;
                            max-height: 60vh;
                            overflow:auto;
                            padding-right: .35rem;
                        "
                    >
                        <Show
                            when=move || messages().is_some()
                            fallback=move || view! { <p style="opacity:.75;">"Loading messages..."</p> }
                        >
                            {move || {
                                messages()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|message| {
                                        let is_me = state.user_id.get() == Some(message.sender);
                                        let sender = display_name(message.sender, &state)
                                            .unwrap_or_else(|| "Unknown".to_string());
                                        let timestamp =
                                            convert_utc_to_local(message.timestamp).format("%H:%M");
                                        view! {
                                            <div
                                                style=move || {
                                                    format!(
                                                        "display:flex; flex-direction:column; align-items:{};",
                                                        if is_me { "flex-end" } else { "flex-start" }
                                                    )
                                                }
                                            >
                                                <div
                                                    style="font-size: 0.85rem; opacity: 0.7; margin-bottom: 0.15rem;"
                                                >
                                                    <span>{sender}</span>
                                                    <span style="margin-left: 0.5rem;">{timestamp.to_string()}</span>
                                                </div>
                                                <div
                                                    class="surface"
                                                    data-accent=if is_me { "base" } else { "weakest" }
                                                    style=move || {
                                                        let align = if is_me { "flex-end" } else { "flex-start" };
                                                        format!(
                                                            "max-width: 70%; padding: 0.6rem 0.75rem; border-radius: 0.75rem; align-self: {align};"
                                                        )
                                                    }
                                                >
                                                    {message.message}
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </Show>
                    </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn display_name(sender: UserId, state: &AppStateReader) -> Option<String> {
    state
        .users
        .get()
        .get(&sender)
        .and_then(|u| u.display_name.clone())
}

fn build_event_card(post_id: PostId, state: &AppStateReader) -> Option<EventCardProps> {
    let post = state.posts.get().get(&post_id)?.clone();
    let author = display_name(post.owner, state).unwrap_or_else(|| "-".to_string());
    Some(EventCardProps {
        id: post_id,
        title: format!("{}", post.title),
        date: peer_practice_shared::convert_utc_to_local_date(post.date)
            .format("%Y-%m-%d")
            .to_string(),
        level: post.level,
        ideas: post.content,
        partaking: post.partaking_users.iter().cloned().collect::<HashSet<_>>(),
        author,
    })
}
