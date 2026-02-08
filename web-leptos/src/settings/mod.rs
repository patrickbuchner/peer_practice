use crate::app_state::AppStateReader;

use leptos::prelude::*;

mod general;

#[component]
pub fn Settings(state: AppStateReader) -> impl IntoView {

    view! { <general::Settings state /> }
}
