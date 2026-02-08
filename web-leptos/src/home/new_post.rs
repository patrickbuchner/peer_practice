use crate::app_state::AppStateReader;
use crate::event_card::{editable::EventCardEditable, EventCardProps};
use leptos::prelude::*;
use crate::components::styles::color::CssVar;

#[component]
pub fn NewPost(state: AppStateReader) -> impl IntoView {
    let read_new_post: ReadSignal<Option<EventCardProps>> = expect_context();
    let write_new_post: WriteSignal<Option<EventCardProps>> = expect_context();

    view! {
        <Show when=move || {
            read_new_post.get().is_some()
        }>
            {move || {
                let props = read_new_post.get().unwrap();
                let (accent_color, _set_accent_teal) = signal(CssVar::Teal.as_str().to_string());
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
    }
}