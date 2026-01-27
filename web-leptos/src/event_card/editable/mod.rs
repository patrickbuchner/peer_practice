use chrono::NaiveDate;
use leptos::prelude::*;
use std::sync::Arc;

use crate::app_state::AppStateReader;
use crate::components::buttons::ConfirmDeleteButton;
use crate::components::buttons::ServerButton;
use crate::components::card::CardForm;
use crate::components::select_input::SelectInput;
use crate::components::text_box::TextBox;
use crate::components::text_input::TextAreaInput;
use crate::components::theme::{AccentStrength, CardShadow, Theme};
use crate::event_card::editable::draft::{Draft, clear_draft, save_draft};
use crate::event_card::{EventCardProps, event_card_footer, markdown_to_safe_html};
use peer_practice_shared::level::Level;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::messages::client_to_server::PostAction;
use peer_practice_shared::post::{PostId, Topics};
use peer_practice_shared::{convert_to_utc, convert_utc_to_local_date, ymd};

mod draft;
#[component]
pub fn EventCardEditable(
    props: EventCardProps,
    #[prop(into)] state: AppStateReader,
    #[prop(optional, into)] accent_color: Option<ReadSignal<String>>,
    #[prop(optional)] on_submitted: Option<Callback<()>>,
) -> impl IntoView {
    let (title, set_title) = signal(props.title.to_string());
    let (level, set_level) = signal(props.level);
    let (ideas, set_ideas) = signal(props.ideas.clone());
    let (show_preview, _set_show_preview) = signal(false);
    let (topics, set_topics) = signal::<Topics>(props.title.as_str().into());

    let accent_color = accent_color.unwrap_or_else(|| {
        let (default_accent, _set_default_accent) =
            signal(String::from("var(--bg-strongest-color)"));
        default_accent
    });

    let ideas_html = Signal::derive(move || markdown_to_safe_html(&ideas.get()));

    let is_new_post = props.id == PostId::NULL;
    let card_theme = if is_new_post { Theme::Accent } else { Theme::Strong };
    let input_theme = card_theme;
    let accent_strength = if is_new_post {
        AccentStrength::Strong
    } else {
        AccentStrength::Base
    };

    let date_options = ymd::create_date_options();
    let initial_date = {
        let first = date_options.first().cloned().unwrap_or_default();
        if !props.date.is_empty() && date_options.contains(&props.date) {
            props.date.clone()
        } else {
            first
        }
    };
    let (date_selected, set_date_selected) = signal(initial_date);
    let post_id = props.id;

    let initial_draft = draft::load_draft(post_id);
    if let Some(d) = initial_draft.clone() {
        set_title.set(d.title);
        set_ideas.set(d.ideas);
        set_level.set(d.level);
        let draft_date = d.date.format("%Y-%m-%d").to_string();
        if date_options.contains(&draft_date) {
            set_date_selected.set(draft_date);
        }
        set_topics.set(title.get().as_str().into());
    }
    let (has_draft, set_has_draft) = signal(initial_draft.is_some());

    Effect::new({
        move |_| {
            let t: Topics = topics.get();
            let i = ideas.get();
            let lv = level.get();
            let date_str = date_selected.get();
            if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                let mut should_save = true;

                if let Some(existing) = state.posts.get().get(&post_id)
                    && t == existing.title
                    && i == existing.content
                    && lv == existing.level
                    && date == convert_utc_to_local_date(existing.date)
                {
                    should_save = false;
                    clear_draft(post_id);
                    set_has_draft.set(false);
                }

                if should_save {
                    let draft = Draft {
                        title: format!("{}", t),
                        ideas: i,
                        level: lv,
                        date,
                    };
                    save_draft(post_id, &draft);
                    set_has_draft.set(true);
                }
            }
        }
    });

    let on_submit = Callback::new(move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Ok(date) = NaiveDate::parse_from_str(&date_selected.get(), "%Y-%m-%d") else {
            return;
        };
        if let Some(existing) = state.posts.get().get(&post_id) {
            let updated = peer_practice_shared::post::Post {
                title: topics.get(),
                content: ideas.get(),
                level: level.get(),
                owner: existing.owner,
                date: convert_to_utc(date),
                partaking_users: existing.partaking_users.clone(),
            };
            state.send(ClientToServer::Post(PostAction::UpdatePost(post_id, updated)));
            clear_draft(post_id);
            set_has_draft.set(false);
        } else {
            let Some(owner) = state.user_id.get() else {
                return;
            };
            let new_post = peer_practice_shared::post::Post {
                title: topics.get(),
                content: ideas.get(),
                level: level.get(),
                owner,
                date: convert_to_utc(date),
                partaking_users: Default::default(),
            };
            state.send(ClientToServer::Post(PostAction::NewPost(new_post)));
            clear_draft(post_id);
            set_has_draft.set(false);
            if let Some(cb) = on_submitted.as_ref() {
                cb.run(());
            }
        }
    });

    view! {
        <CardForm
            data_theme=card_theme
            data_shadow=CardShadow::Weak
            data_accent=AccentStrength::Strong
            data_accent_strength=accent_strength
            accent_color=accent_color
            on_submit=on_submit
        >
            <div class="cluster cluster--between cluster--gap-sm cluster--nowrap event-card-header">
                <SelectInput
                    class="card-title-input event-card-title-select".to_string()
                    data_theme=input_theme
                    data_accent_strength=accent_strength
                    accent_color=accent_color
                    value=Signal::derive(move || topics.get().to_string())
                    on_change=Callback::new(move |ev| {
                        let v = event_target_value(&ev);
                        set_topics.set(v.as_str().into());
                        set_title.set(v);
                    })
                >
                    {Topics::ALL
                        .iter()
                        .map(|t| {
                            let v = t.to_string();
                            let label = t.to_string();
                            view! { <option value=v.clone()>{label}</option> }
                        })
                        .collect_view()}
                </SelectInput>

                <SelectInput
                    class="event-card-date-select".to_string()
                    data_theme=input_theme
                    data_accent_strength=accent_strength
                    accent_color=accent_color
                    value=Signal::derive(move || date_selected.get())
                    on_change=Callback::new(move |ev| set_date_selected.set(event_target_value(&ev)))
                >
                    {date_options
                        .iter()
                        .cloned()
                        .map(|d| {
                            let d_clone = d.clone();
                            view! { <option value=d_clone>{d}</option> }
                        })
                        .collect_view()}
                </SelectInput>
            </div>

            <div class="cluster cluster--start cluster--gap-md cluster--nowrap event-card-row">
                <span class="event-card-label">
                    "Level"
                </span>

                <SelectInput
                    class="event-card-level-select".to_string()
                    data_theme=input_theme
                    data_accent_strength=accent_strength
                    accent_color=accent_color
                    value=Signal::derive(move || level.get().as_str().to_string())
                    on_change=Callback::new(move |ev| {
                        let v = event_target_value(&ev);
                        set_level.set(Level::from(v.as_str()));
                    })
                >
                    {Level::all()
                        .iter()
                        .map(|lv| {
                            let v = lv.as_str().to_string();
                            let label = lv.to_string();
                            view! { <option value=v.clone()>{label}</option> }
                        })
                        .collect_view()}
                </SelectInput>
            </div>

            <div class="cluster cluster--start cluster--gap-md event-card-row">
                <span class="event-card-label">"Ideas"</span>
                // <button
                // class="btn btn--icon"
                // data-theme="ghost"
                // aria-pressed=move || show_preview.get().to_string()
                // on:click=move |_| set_show_preview.update(|v| *v = !*v)
                // style="flex: 0 0 auto;"
                // type="button"
                // >
                // <svg
                // width="18"
                // height="18"
                // viewBox="0 0 24 24"
                // role="img"
                // aria-hidden="true"
                // focusable="false"
                // style="display:block"
                // >
                // <circle
                // cx="12"
                // cy="12"
                // r="8"
                // fill="none"
                // stroke="currentColor"
                // stroke-width="2"
                // />
                // <Show when=move || !show_preview.get()>
                // <circle cx="12" cy="12" r="4.25" fill="currentColor" />
                // </Show>
                // </svg>
                // </button>
                <div
                    class="event-card-ideas-grid"
                    style=move || {
                        let cols = if show_preview.get() { "1fr 1fr" } else { "1fr" };
                        format!("--ideas-columns: {};", cols)
                    }
                >
                    <TextAreaInput
                        class="event-card-textarea".to_string()
                        data_theme=input_theme
                        data_accent_strength=accent_strength
                        accent_color=accent_color
                        value=Signal::derive(move || ideas.get())
                        on_input=Callback::new(move |ev| {
                            set_ideas.set(event_target_value(&ev));
                        })
                    />
                    <Show when=move || show_preview.get()>
                        <TextBox
                            class="event-card-preview".to_string()
                            role="region".to_string()
                            aria_label="Live preview".to_string()
                            data_theme=input_theme
                            data_accent_strength=accent_strength
                            accent_color=accent_color
                            html=ideas_html
                        />
                    </Show>
                </div>
            </div>

            {event_card_footer(props, state)}

            <div class="event-card-actions">
                <div></div>

                <ServerButton
                    class=Signal::derive(|| "btn btn--sm".to_string())
                    data_theme=Arc::new(|| Theme::Secondary)
                    r#type="submit".to_string()
                >
                    "Submit"
                </ServerButton>

                <div class="event-card-actions-end">
                    <div class="cluster cluster--gap-sm cluster--align-center">
                        <Show when=move || {
                            has_draft.get() && state.posts.get().contains_key(&post_id)
                        }>
                            <button
                                class="btn btn--sm"
                                data-theme=Theme::Secondary.as_str()
                                type="button"
                                title="Reset to server version (discard local draft)"
                                on:click=move |_| {
                                    if let Some(existing) = state.posts.get().get(&post_id) {
                                        set_title.set(format!("{}", existing.title));
                                        set_ideas.set(existing.content.clone());
                                        set_level.set(existing.level);
                                        let d = convert_utc_to_local_date(existing.date)
                                            .format("%Y-%m-%d")
                                            .to_string();
                                        set_date_selected.set(d);
                                        set_topics.set(existing.title);
                                        clear_draft(post_id);
                                        set_has_draft.set(false);
                                    }
                                }
                            >
                                "Reset"
                            </button>
                        </Show>
                        <Show when=move || { state.posts.get().contains_key(&post_id) }>
                            <ConfirmDeleteButton
                                button_label="🗑️".to_string()
                                button_title="Delete post".to_string()
                                confirm_title="Delete this post?".to_string()
                                confirm_message="This action cannot be undone.".to_string()
                                on_confirm=Callback::new({
                                    move |_| {
                                        state.send(ClientToServer::Post(PostAction::DeletePost(post_id)));
                                    }
                                })
                            />
                        </Show>
                    </div>
                </div>
            </div>
        </CardForm>
    }
}
