use chrono::NaiveDate;
use leptos::prelude::*;
use std::sync::Arc;

use crate::app_state::AppStateReader;
use crate::components::buttons::ConfirmDeleteButton;
use crate::components::buttons::ServerButton;
use crate::components::card::CardForm;
use crate::components::select_input::SelectInput;
use crate::components::styles::button_class::ButtonClass;
use crate::components::styles::cluster::ClusterClass;
use crate::components::styles::color::CssVar;
use crate::components::styles::event_card::EventCardClass;
use crate::components::styles::ideas::IdeasColumns;
use crate::components::text_box::TextBox;
use crate::components::text_input::TextAreaInput;
use crate::components::theme::{AccentStrength, CardShadow, IntentTheme, SurfaceTheme, Theme};
use crate::event_card::editable::draft::{Draft, clear_draft, save_draft};
use crate::event_card::{
    EventCardProps, event_card_footer, markdown_to_safe_html, shadow_color_for_date,
};
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

    let is_new_post = props.id == PostId::NULL;
    let theme = if is_new_post {
        Theme::Accent
    } else {
        Theme::Surface(SurfaceTheme::Strong)
    };
    let accent_strength = if is_new_post {
        AccentStrength::Strong
    } else {
        AccentStrength::Base
    };
    let accent_color = accent_color.unwrap_or_else(|| {
        let default = if is_new_post {
            CssVar::Teal.as_str()
        } else {
            CssVar::BgStrong.as_str()
        };
        let (default_accent, _set_default_accent) = signal(default.to_string());
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
    let (shadow_color, set_shadow_color) = signal(if is_new_post {
        crate::components::styles::color::ShadowColor::Teal
    } else {
        shadow_color_for_date(&date_selected.get_untracked())
    });
    Effect::new(move |_| {
        let next = if is_new_post {
            crate::components::styles::color::ShadowColor::Teal
        } else {
            shadow_color_for_date(&date_selected.get())
        };
        set_shadow_color.set(next);
    });

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
            state.send(ClientToServer::Post(PostAction::UpdatePost(
                post_id, updated,
            )));
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
            data_theme=theme
            data_shadow=CardShadow::Weakest
            shadow_color=shadow_color
            data_accent_strength=accent_strength
            accent_color=accent_color
            on_submit=on_submit
        >
            <div class=EventCardClass::Header.as_str()>
                <SelectInput
                    class=EventCardClass::TitleSelect.as_str().to_string()
                    data_theme=theme
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
                    class=EventCardClass::DateSelect.as_str().to_string()
                    data_theme=theme
                    data_accent_strength=accent_strength
                    accent_color=accent_color
                    value=Signal::derive(move || date_selected.get())
                    on_change=Callback::new(move |ev| {
                        set_date_selected.set(event_target_value(&ev))
                    })
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

            <div class=EventCardClass::RowNoWrap.as_str()>
                <span class=EventCardClass::Label.as_str()>"Level"</span>

                <SelectInput
                    class=EventCardClass::LevelSelect.as_str().to_string()
                    data_theme=theme
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

            <div class=EventCardClass::Row.as_str()>
                <span class=EventCardClass::Label.as_str()>"Ideas"</span>
                <div
                    class=EventCardClass::IdeasGrid.as_str()
                    style=move || {
                        if show_preview.get() {
                            IdeasColumns::Split.to_style()
                        } else {
                            IdeasColumns::Single.to_style()
                        }
                    }
                >
                    <TextAreaInput
                        class=EventCardClass::Textarea.as_str().to_string()
                        data_theme=theme
                        data_accent_strength=accent_strength
                        accent_color=accent_color
                        value=Signal::derive(move || ideas.get())
                        on_input=Callback::new(move |ev| {
                            set_ideas.set(event_target_value(&ev));
                        })
                    />
                    <Show when=move || show_preview.get()>
                        <TextBox
                            class=EventCardClass::Preview.as_str().to_string()
                            role="region".to_string()
                            aria_label="Live preview".to_string()
                            data_theme=theme
                            data_accent_strength=accent_strength
                            accent_color=accent_color
                            html=ideas_html
                        />
                    </Show>
                </div>
            </div>

            {event_card_footer(props, state)}

            <div class=EventCardClass::Actions.as_str()>
                <div></div>

                <ServerButton
                    class=Signal::derive(|| ButtonClass::Small.as_str().to_string())
                    data_theme=Arc::new(|| Theme::Intent(IntentTheme::Secondary))
                    r#type="submit".to_string()
                >
                    "Submit"
                </ServerButton>

                <div class=EventCardClass::ActionsEnd.as_str()>
                    <div class=ClusterClass::GapSmAlignCenter.as_str()>
                        <Show when=move || {
                            has_draft.get() && state.posts.get().contains_key(&post_id)
                        }>
                            <button
                                class=ButtonClass::Small.as_str()
                                data-theme=Theme::Intent(IntentTheme::Secondary).as_str()
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
                                        state
                                            .send(
                                                ClientToServer::Post(PostAction::DeletePost(post_id)),
                                            );
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
