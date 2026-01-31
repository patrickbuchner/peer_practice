use crate::app_state::AppStateReader;
use crate::components::card::Card;
use crate::components::styles::button_class::ButtonClass;
use crate::components::styles::chat::ChatClass;
use crate::components::styles::cluster::ClusterClass;
use crate::components::styles::color::{chat_border_style, chat_name_style};
use crate::components::styles::layout::LayoutClass;
use crate::components::styles::text_class::TextClass;
use crate::components::text_input::TextInput;
use crate::components::theme::{AccentStrength, CardShadow, IntentTheme, SurfaceTheme, Theme};
use crate::event_card::{EventCardProps, readonly::EventCardReadonly};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use peer_practice_shared::chat::{ChatId, ChatMessage, ChatMessageKind};
use peer_practice_shared::colors::accent_colors::AccentColor;
use peer_practice_shared::convert_utc_to_local;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::messages::client_to_server::ChatAction;
use peer_practice_shared::post::PostId;
use peer_practice_shared::user::UserId;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::window;

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
    let (draft, set_draft) = signal(String::new());
    let can_send = Signal::derive(move || !draft.get().trim().is_empty());
    let chat_ref = NodeRef::<leptos::html::Div>::new();
    let (auto_scroll, set_auto_scroll) = signal(true);

    let handle_scroll = move |_| {
        if let Some(node) = chat_ref.get() {
            let scroll_top = node.scroll_top() as f64;
            let client_height = node.client_height() as f64;
            let scroll_height = node.scroll_height() as f64;
            let near_bottom = scroll_top + client_height >= scroll_height - 24.0;
            set_auto_scroll.set(near_bottom);
        }
    };

    Effect::new(move |_| {
        let _ = messages().map(|msgs| msgs.len());
        if !auto_scroll.get() {
            return;
        }
        if let Some(node) = chat_ref.get() {
            let node = node.clone();
            let _ = window().and_then(|win| {
                let cb = Closure::once(move || {
                    let height = node.scroll_height();
                    node.set_scroll_top(height);
                });
                win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    0,
                )
                .ok()?;
                cb.forget();
                Some(())
            });
        }
    });

    view! {
        <div class=LayoutClass::PagePadStack.as_str()>
            <Show
                when=has_valid_chat_id
                fallback=move || {
                    view! {
                        <Card
                            data_theme=Theme::Surface(SurfaceTheme::Strong)
                            data_shadow=CardShadow::Weakest
                        >
                            <h3 class=TextClass::CardTitle.as_str()>"Invalid chat id"</h3>
                            <p class=TextClass::CardNoteMutedSm
                                .as_str()>"The chat id in the URL could not be parsed."</p>
                        </Card>
                    }
                }
            >
                <div>
                    <Show
                        when=move || associated_post().is_some()
                        fallback=move || {
                            view! {
                                <Card
                                    data_theme=Theme::Surface(SurfaceTheme::Strong)
                                    data_shadow=CardShadow::Weakest
                                >
                                    <h3 class=TextClass::CardTitle
                                        .as_str()>"Associated post unavailable"</h3>
                                    <p class=TextClass::CardNoteMutedSm
                                        .as_str()>
                                        "We could not find a post linked to this chat yet."
                                    </p>
                                </Card>
                            }
                        }
                    >
                        {move || {
                            associated_post()
                                .map(|props| view! { <EventCardReadonly props state /> })
                        }}
                    </Show>
                </div>

                <div>
                    <Card
                        data_theme=Theme::Surface(SurfaceTheme::Strong)
                        data_shadow=CardShadow::Weakest
                    >
                        <div class=ClusterClass::BetweenGapSmAlignCenter.as_str()>
                            <h3 class=TextClass::CardTitle.as_str()>"Chat"</h3>
                            <span class=TextClass::DimSm
                                .as_str()>
                                {move || messages().map(|m| m.len()).unwrap_or(0)} " messages"
                            </span>
                        </div>

                        <div
                            class=ChatClass::Messages.as_str()
                            node_ref=chat_ref
                            on:scroll=handle_scroll
                        >
                            <Show
                                when=move || messages().is_some()
                                fallback=move || {
                                    view! {
                                        <p class=TextClass::Dim.as_str()>"Loading messages..."</p>
                                    }
                                }
                            >
                                {move || {
                                    let list = messages().unwrap_or_default();
                                    let mut latest_system: std::collections::HashMap<
                                        UserId,
                                        usize,
                                    > = std::collections::HashMap::new();
                                    for (idx, msg) in list.iter().enumerate() {
                                        if !matches!(msg.kind, ChatMessageKind::Text(_)) {
                                            latest_system.insert(msg.sender, idx);
                                        }
                                    }
                                    list.into_iter()
                                        .enumerate()
                                        .filter_map(|(idx, message)| {
                                            let is_me = state.user_id.get() == Some(message.sender);
                                            let sender = display_name(message.sender, &state)
                                                .unwrap_or_else(|| "Unknown".to_string());
                                            let timestamp = convert_utc_to_local(message.timestamp)
                                                .format("%H:%M");
                                            let accent_color = chat_accent_color(
                                                message.chat_id,
                                                message.sender,
                                            );
                                            let view = match message.kind {
                                                ChatMessageKind::Text(text) => {
                                                    view! {
                                                        <div class=if is_me {
                                                            ChatClass::MessageMine.as_str()
                                                        } else {
                                                            ChatClass::Message.as_str()
                                                        }>
                                                            <div class=ChatClass::Meta.as_str()>
                                                                <span style=chat_name_style(accent_color)>{sender}</span>
                                                                <span>{timestamp.to_string()}</span>
                                                            </div>
                                                            <div
                                                                class=ChatClass::BubbleSurface.as_str()
                                                                style=chat_border_style(accent_color)
                                                                data-accent=if is_me {
                                                                    AccentStrength::Base.as_str()
                                                                } else {
                                                                    AccentStrength::Weak.as_str()
                                                                }
                                                                data-theme=Theme::Surface(SurfaceTheme::Strong).as_str()
                                                            >
                                                                {text}
                                                            </div>
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                                ChatMessageKind::Joined => {
                                                    if latest_system.get(&message.sender).copied() != Some(idx)
                                                    {
                                                        return None;
                                                    }
                                                    let text = format!("{sender} joined");
                                                    view! {
                                                        <div class=ChatClass::MessageSystem.as_str()>
                                                            <div class=ChatClass::BubbleSystem.as_str()>
                                                                <span>{text}</span>
                                                                <span>{timestamp.to_string()}</span>
                                                            </div>
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                                ChatMessageKind::Left => {
                                                    if latest_system.get(&message.sender).copied() != Some(idx)
                                                    {
                                                        return None;
                                                    }
                                                    let text = format!("{sender} left");
                                                    view! {
                                                        <div class=ChatClass::MessageSystem.as_str()>
                                                            <div class=ChatClass::BubbleSystem.as_str()>
                                                                <span>{text}</span>
                                                                <span>{timestamp.to_string()}</span>
                                                            </div>
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            };
                                            Some(view)
                                        })
                                        .collect_view()
                                }}
                            </Show>
                        </div>
                        <form
                            class=ChatClass::InputBar.as_str()
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let text = draft.get().trim().to_string();
                                let Some(chat_id) = chat_id.get() else {
                                    return;
                                };
                                let Some(sender) = state.user_id.get() else {
                                    return;
                                };
                                if text.is_empty() {
                                    return;
                                }
                                state
                                    .send(
                                        ClientToServer::Chat(
                                            ChatAction::SendMessage(ChatMessage {
                                                sender,
                                                kind: ChatMessageKind::Text(text),
                                                chat_id,
                                            }),
                                        ),
                                    );
                                set_draft.set(String::new());
                            }
                        >
                            <TextInput
                                r#type="text".to_string()
                                class=ChatClass::InputField.as_str().to_string()
                                placeholder="Write a message...".to_string()
                                value=Signal::derive(move || draft.get())
                                on_input=Callback::new(move |ev| {
                                    set_draft.set(event_target_value(&ev));
                                })
                                data_theme=Theme::Surface(SurfaceTheme::Strong)
                            />
                            <button
                                class=ButtonClass::Base.as_str()
                                data-theme=Theme::Intent(IntentTheme::Primary).as_str()
                                type="submit"
                                aria-disabled=move || (!can_send.get()).to_string()
                            >
                                "Send"
                            </button>
                        </form>
                    </Card>
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

fn chat_accent_color(chat_id: ChatId, sender: UserId) -> AccentColor {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    chat_id.hash(&mut hasher);
    sender.hash(&mut hasher);
    let idx = (hasher.finish() as usize) % AccentColor::base().len();
    AccentColor::base()[idx]
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
