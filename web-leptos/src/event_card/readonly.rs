use crate::app_state::AppStateReader;
use crate::components::card::Card;
use crate::components::text_box::{SurfaceBox, TextBox};
use crate::components::theme::{AccentStrength, CardShadow, Theme};
use crate::event_card::{EventCardProps, event_card_footer, markdown_to_safe_html};
use leptos::prelude::*;

#[component]
pub fn EventCardReadonly(
    props: EventCardProps,
    #[prop(into)] state: AppStateReader,
    #[prop(optional, into)] accent_color: Option<ReadSignal<String>>,
) -> impl IntoView {
    let ideas = props.ideas.clone();
    let ideas_html = Signal::derive(move || markdown_to_safe_html(&ideas));
    let accent_color = accent_color.unwrap_or_else(|| {
        let (default_accent, _set_default_accent) =
            signal(String::from("var(--bg-strongest-color)"));
        default_accent
    });

    view! {
        <Card
            data_theme=Theme::Strong
            data_shadow=CardShadow::Strong
            data_accent=AccentStrength::Strong
            accent_color=accent_color
        >
            <div class="cluster cluster--between cluster--gap-sm">
                <h3 class="card-title">{props.title.clone()}</h3>
                <span class="text-muted">{props.date.clone()}</span>
            </div>

            <div
                class="cluster cluster--start cluster--gap-md event-card-row"
            >
                <span class="event-card-label">"Level"</span>
                <SurfaceBox class="event-card-badge".to_string()>
                    {props.level.to_string()}
                </SurfaceBox>
            </div>

            <div
                class="cluster cluster--start cluster--gap-md event-card-row"
            >
                <span class="event-card-label">"Ideas"</span>
                <TextBox
                    class="event-card-ideas".to_string()
                    data_theme=Theme::Strong
                    accent_color=accent_color
                    html=ideas_html
                />
            </div>

            {event_card_footer(props, state)}
        </Card>
    }
}
