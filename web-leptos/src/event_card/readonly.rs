use crate::app_state::AppStateReader;
use crate::components::card::Card;
use crate::components::styles::color::CssVar;
use crate::components::text_box::{SurfaceBox, TextBox};
use crate::components::theme::{CardShadow, Theme};
use crate::event_card::{event_card_footer, markdown_to_safe_html, shadow_color_for_date, EventCardProps};
use leptos::prelude::*;
use crate::components::styles::cluster::ClusterClass;
use crate::components::styles::event_card::EventCardClass;
use crate::components::styles::text_class::TextClass;

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
            signal(CssVar::BgStrong.as_str().to_string());
        default_accent
    });
    let shadow_color = {
        let (read, _set) = signal(shadow_color_for_date(&props.date));
        read
    };

    let theme = Theme::Strong;

    view! {
        <Card
            data_theme=theme
            data_shadow=CardShadow::Weakest
            shadow_color=shadow_color
            accent_color=accent_color
        >
            <div class=ClusterClass::BetweenGapSm.as_str()>
                <h3 class=TextClass::CardTitle.as_str()>{props.title.clone()}</h3>
                <span class=TextClass::Muted.as_str()>{props.date.clone()}</span>
            </div>

            <div class=EventCardClass::Row.as_str()>
                <span class=EventCardClass::Label.as_str()>"Level"</span>
                <SurfaceBox class=EventCardClass::Badge.as_str().to_string()>
                    {props.level.to_string()}
                </SurfaceBox>
            </div>

            <div class=EventCardClass::Row.as_str()>
                <span class=EventCardClass::Label.as_str()>"Ideas"</span>
                <TextBox
                    class=EventCardClass::Ideas.as_str().to_string()
                    data_theme=theme
                    accent_color=accent_color
                    html=ideas_html
                />
            </div>

            {event_card_footer(props, state)}
        </Card>
    }
}
