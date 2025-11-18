use crate::app_state::AppStateReader;
use crate::event_card::{EventCardProps, event_card_footer, markdown_to_safe_html};
use leptos::prelude::*;

#[component]
pub fn EventCardReadonly(
    props: EventCardProps,
    #[prop(into)] state: AppStateReader,
    #[prop(optional, into)] accent_color: Option<ReadSignal<String>>,
) -> impl IntoView {
    let ideas_html = markdown_to_safe_html(&props.ideas);
    let accent_color = accent_color.unwrap_or_else(|| {
        let (default_accent, _set_default_accent) =
            signal(String::from("var(--bg-strongest-color)"));
        default_accent
    });

    view! {
        <div
            class="card"
            data-accent="base"
            style=move || { format!("--accent: {};", accent_color.get()) }
        >
            <div class="cluster" style="--cluster-justify: space-between; --cluster-gap: .5rem;">
                <h3 class="card-title">{props.title.clone()}</h3>
                <span style="opacity: .85;">{props.date.clone()}</span>
            </div>

            <div
                class="cluster"
                style="--cluster-justify: flex-start; --cluster-gap: .75rem; margin-top: .75rem;"
            >
                <span style="min-width: 3rem; text-align: left; opacity: .8;">"Level"</span>
                <div
                    class="surface"
                    data-accent="base"
                    style=move || {
                        format!(
                            "--accent: {}; display:inline-block; min-width: 12rem; padding: 0.5rem 0.75rem; border-radius: 0.5rem;",
                            accent_color.get(),
                        )
                    }
                >
                    {props.level.to_string()}
                </div>
            </div>

            <div
                class="cluster"
                style="--cluster-justify: flex-start; --cluster-gap: .75rem; margin-top: .75rem;"
            >
                <span style="min-width: 3rem; text-align: left; opacity: .8;">"Ideas"</span>
                <div
                    class="markdown-body surface"
                    data-accent="base"
                    style=move || {
                        format!(
                            "--accent: {}; flex: 1 1 auto; min-height: 7rem; padding: .75rem; border-radius: .6rem; overflow:auto;",
                            accent_color.get(),
                        )
                    }
                    inner_html=ideas_html
                />
            </div>

            {event_card_footer(props, state)}
        </div>
    }
}
