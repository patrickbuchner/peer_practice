use crate::app_state::AppStateReader;

use leptos::prelude::*;

mod general;
mod sessions;

#[component]
pub fn Settings(state: AppStateReader) -> impl IntoView {

    view! {
        <general::Settings state />
        <sessions::Settings state />
    }
}
