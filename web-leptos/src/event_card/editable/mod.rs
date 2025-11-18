use chrono::NaiveDate;
use leptos::prelude::*;
use std::sync::Arc;

use crate::app_state::AppStateReader;
use crate::components::buttons::ConfirmDeleteButton;
use crate::components::buttons::ServerButton;
use crate::event_card::editable::draft::{Draft, clear_draft, save_draft};
use crate::event_card::{EventCardProps, event_card_footer, markdown_to_safe_html};
use peer_practice_shared::level::Level;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::post::Topics;
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

    view! {
        <form
            class="card"
            data-accent="base"
            data-scheme="base"
            style=move || { format!("--accent: {};", accent_color.get()) }
            on:submit=move |ev| {
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
                    state.send(ClientToServer::UpdatePost(post_id, updated));
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
                    state.send(ClientToServer::NewPost(new_post));
                    clear_draft(post_id);
                    set_has_draft.set(false);
                    if let Some(cb) = on_submitted {
                        cb.run(());
                    }
                }
            }
        >
            <div
                class="cluster"
                style="\
                --cluster-justify: space-between; --cluster-gap: .5rem; \
                flex-wrap: nowrap; align-items: center; \
                "
            >
                <select
                    class="combo card-title-input border-round"
                    data-theme="accent"
                    data-accent-strength="base"
                    style=move || {
                        format!("flex: 1 1 8rem; min-width: 0; --accent: {};", accent_color.get())
                    }
                    prop:value=move || topics.get().to_string()
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        set_topics.set(v.as_str().into());
                        set_title.set(v);
                    }
                >
                    {Topics::ALL
                        .iter()
                        .map(|t| {
                            let v = t.to_string();
                            let label = t.to_string();
                            view! { <option value=v.clone()>{label}</option> }
                        })
                        .collect_view()}
                </select>

                <select
                    class="combo"
                    data-theme="accent"
                    data-accent-strength="base"
                    style=move || {
                        format!(
                            "flex: 0 0 auto; width: auto; max-width: 12rem; --accent: {};",
                            accent_color.get(),
                        )
                    }
                    prop:value=move || date_selected.get()
                    on:change=move |ev| set_date_selected.set(event_target_value(&ev))
                >
                    {date_options
                        .iter()
                        .cloned()
                        .map(|d| {
                            let d_clone = d.clone();
                            view! { <option value=d_clone>{d}</option> }
                        })
                        .collect_view()}
                </select>
            </div>

            <div
                class="cluster"
                style="\
                --cluster-justify: flex-start; --cluster-gap: .75rem; margin-top: .75rem; \
                flex-wrap: nowrap; \
                "
            >
                <span style="flex: 0 0 auto; min-width: 3rem; text-align: left; opacity: .8;">
                    "Level"
                </span>

                <select
                    class="combo"
                    data-theme="accent"
                    data-accent-strength="base"
                    style=move || {
                        format!(
                            "flex: 1 1 auto; min-width: 0; max-width: 100%; --accent: {};",
                            accent_color.get(),
                        )
                    }
                    prop:value=move || level.get().as_str().to_string()
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        set_level.set(Level::from(v.as_str()));
                    }
                >
                    {Level::all()
                        .iter()
                        .map(|lv| {
                            let v = lv.as_str().to_string();
                            let label = lv.to_string();
                            view! { <option value=v.clone()>{label}</option> }
                        })
                        .collect_view()}
                </select>
            </div>

            <div
                class="cluster"
                style="--cluster-justify: flex-start; --cluster-gap: .75rem; margin-top: .75rem;"
            >
                <span style="min-width: 3rem; text-align: left; opacity: .8;">"Ideas"</span>
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
                <div style=move || {
                    let cols = if show_preview.get() { "1fr 1fr" } else { "1fr" };
                    format!(
                        "display: grid; grid-template-columns: {}; gap: .75rem; width: 100%;",
                        cols,
                    )
                }>
                    <textarea
                        class="surface"
                        data-accent="base"
                        style=move || {
                            format!(
                                "--accent: {}; min-height: 7rem; resize: vertical; padding: .75rem; border-radius: .6rem;",
                                accent_color.get(),
                            )
                        }
                        prop:value=move || ideas.get()
                        on:input=move |ev| set_ideas.set(event_target_value(&ev))
                    />
                    <Show when=move || show_preview.get()>
                        <div
                            class="markdown-body"
                            data-theme="accent"
                            role="region"
                            aria-label="Live preview"
                            style=move || {
                                format!(
                                    "--accent: {}; min-height: 7rem; padding: .75rem; border-radius: .6rem; overflow: auto;",
                                    accent_color.get(),
                                )
                            }
                            inner_html=ideas_html
                        />
                    </Show>
                </div>
            </div>

            {event_card_footer(props, state)}

            <div
                class="cluster"
                style="display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; margin-top: .75rem;"
            >
                <div></div>

                <ServerButton
                    class=Signal::derive(|| "btn".to_string())
                    data_theme=Arc::new(|| "secondary")
                    style="font-size: .9rem; padding: .35rem .6rem;".to_string()
                    r#type="submit".to_string()
                >
                    "Submit"
                </ServerButton>

                <div style="justify-self: end;">
                    <div class="cluster" style="--cluster-gap: .5rem; align-items: center;">
                        <Show when=move || {
                            has_draft.get() && state.posts.get().contains_key(&post_id)
                        }>
                            <button
                                class="btn"
                                data-theme="secondary"
                                style="font-size: .9rem; padding: .35rem .6rem;"
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
                                        state.send(ClientToServer::DeletePost(post_id));
                                    }
                                })
                            />
                        </Show>
                    </div>
                </div>
            </div>
        </form>
    }
}
