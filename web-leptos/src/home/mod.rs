use crate::app_state::AppStateReader;
use leptos::prelude::*;
use crate::home::new_post::NewPost;
use crate::home::normal_posts::PracticeIdeas;

mod new_post;
mod normal_posts;

#[component]
pub fn Home(#[prop(into)] state: AppStateReader) -> impl IntoView {
    view! {
        <div>
            <NewPost state />
            <PracticeIdeas state />
        </div>
    }
}

