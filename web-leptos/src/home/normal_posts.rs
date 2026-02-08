use crate::app_state::AppStateReader;
use crate::components::styles::event_card::EventListClass;
use crate::components::styles::stack::StackStyle;
use crate::event_card::EventCardProps;
use crate::event_card::editable::EventCardEditable;
use crate::event_card::readonly::EventCardReadonly;
use leptos::prelude::*;
use peer_practice_shared::convert_utc_to_local_date;
use peer_practice_shared::post::{Post, PostId};
use peer_practice_shared::user::UserId;
use peer_practice_shared::user::display_user::UserDisplay;
use std::collections::{HashMap, HashSet};

#[component]
pub fn PracticeIdeas(state: AppStateReader) -> impl IntoView {
    let items_view = move || {
        let items = create_card_descriptions(state.posts.get(), state.users.get());
        let mut last_date: Option<String> = None;
        let total = items.len();

        items
            .into_iter()
            .enumerate()
            .map(|(idx, (owner, props))| {
                let date = props.date.clone();
                let gap = get_gap(&last_date, &props);
                last_date = Some(date);
                let card_view = create_card_view(owner, props, state);
                let stack_index = total.saturating_sub(idx);
                view! {
                    <div class=gap.as_str() style=StackStyle::Card.with_z_index(stack_index)>
                        {card_view}
                    </div>
                }
                .into_any()
            })
            .collect::<Vec<_>>()
    };

    view! { {items_view} }
}

fn get_gap(last_date: &Option<String>, props: &EventCardProps) -> EventListClass {
    if last_date.as_ref().is_some_and(|prev| prev != &props.date) {
        EventListClass::DateGap
    } else {
        EventListClass::None
    }
}

fn create_card_view(owner: UserId, props: EventCardProps, state: AppStateReader) -> impl IntoView {
    if Some(owner) == state.user_id.get() {
        view! { <EventCardEditable props state /> }.into_any()
    } else {
        view! { <EventCardReadonly props state /> }.into_any()
    }
}

fn create_card_descriptions(
    posts: HashMap<PostId, Post>,
    users: HashMap<UserId, UserDisplay>,
) -> Vec<(UserId, EventCardProps)> {
    let mut items = posts
        .iter()
        .map(|(&id, post)| {
            (
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
                    author: users
                        .get(&post.owner)
                        .and_then(|u| u.display_name.clone())
                        .unwrap_or_else(|| "-".to_string()),
                },
            )
        })
        .collect::<Vec<(UserId, EventCardProps)>>();

    items.sort_by(|a, b| {
        a.1.date
            .cmp(&b.1.date)
            .then_with(|| a.1.title.cmp(&b.1.title))
    });

    items
}
