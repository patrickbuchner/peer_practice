use crate::app_state::AppStateReader;
use crate::event_card::{EventCardProps, editable::EventCardEditable, readonly::EventCardReadonly};
use leptos::prelude::*;
use peer_practice_shared::convert_utc_to_local_date;
use peer_practice_shared::user::UserId;
use std::collections::HashSet;

#[component]
pub fn Home(#[prop(into)] state: AppStateReader) -> impl IntoView {
    let read_new_post: ReadSignal<Option<EventCardProps>> = expect_context();
    let write_new_post: WriteSignal<Option<EventCardProps>> = expect_context();

    view! {
        <div>
            <Show when=move || {
                read_new_post.get().is_some()
            }>
                {move || {
                    let props = read_new_post.get().unwrap();
                    let (accent_color, _set_accent_teal) = signal(String::from("var(--teal)"));
                    view! {
                        <EventCardEditable
                            props
                            state
                            accent_color
                            on_submitted=Callback::new({
                                let write_new_post = write_new_post;
                                move |_| {
                                    write_new_post.set(None);
                                }
                            })
                        />
                    }
                }}
            </Show>

            {move || {
                let current_user = state.user_id.get();
                let mut items = state
                    .posts
                    .get()
                    .iter()
                    .map(|(&id, post)| (
                        post.owner,
                        EventCardProps {
                            id,
                            title: format!("{}", post.title),
                            date: convert_utc_to_local_date(post.date)
                                .format("%Y-%m-%d")
                                .to_string(),
                            level: post.level,
                            ideas: post.content.clone(),
                            partaking: post.partaking_users.iter().cloned().collect::<HashSet<_>>(),
                            author: state
                                .users
                                .get()
                                .get(&post.owner)
                                .and_then(|u| u.display_name.clone())
                                .unwrap_or_else(|| "-".to_string()),
                        },
                    ))
                    .collect::<Vec<(UserId, EventCardProps)>>();
                items
                    .sort_by(|a, b| {
                        a.1.date.cmp(&b.1.date).then_with(|| a.1.title.cmp(&b.1.title))
                    });
                items
                    .into_iter()
                    .map(|(owner, props)| {
                        if Some(owner) == current_user {

                            view! { <EventCardEditable props state /> }
                                .into_any()
                        } else {
                            view! { <EventCardReadonly props state /> }.into_any()
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
