use leptos::prelude::*;
use crate::components::styles::{ButtonClass, CardClass, ClusterClass, FormClass, LayoutClass, PaletteStyle, TextClass};
use crate::components::text_input::TextInput;
use std::sync::Arc;

use crate::app_state::AppStateReader;
use crate::components::buttons::ServerButton;
use crate::components::modal::CenterModal;
use crate::components::theme::{CardShadow, Theme};
use peer_practice_shared::accent_colors::AccentColor;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::user::display_user::UserDisplay;

#[component]
pub fn Settings(state: AppStateReader) -> impl IntoView {
    let initial_name = {
        if let Some(uid) = state.user_id.get_untracked() {
            state
                .users
                .get_untracked()
                .get(&uid)
                .and_then(|u| u.display_name.clone())
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    let (name, set_name) = signal(initial_name);
    let (saving, set_saving) = signal(false);

    use peer_practice_shared::messages::client_to_server::UserAction;

    let (accent_color, set_accent_color) = signal(AccentColor::Teal);
    let accent_css = {
        let (ro, set) = signal(accent_color.get_untracked().css_var().to_string());
        Effect::new(move |_| {
            set.set(accent_color.get().css_var().to_string());
        });
        ro
    };
    let (show_palette, set_show_palette) = signal(false);

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        set_saving.set(true);
        let new_name = name.get();
        let id = state.user_id.get().unwrap();
        state.send(ClientToServer::User(UserAction::Update(UserDisplay {
            id,
            display_name: Some(new_name.clone()),
        })));

        set_saving.set(false);
    };

    view! {
        <section class=LayoutClass::ContainerNarrowPadSm.as_str()>
            <div
                class=CardClass::Base.as_str()
                data-theme=Theme::Strong.as_str()
                data-shadow=CardShadow::Weakest.as_str()
            >
                <h2 class=TextClass::CardTitle.as_str()>"Settings"</h2>
                <form class=FormClass::Form.as_str() on:submit=on_submit>
                    <div class=FormClass::Grid.as_str()>
                        <label for="display_name" class=FormClass::LabelEnd.as_str()>
                            "Display name"
                        </label>
                        <TextInput
                            id="display_name".to_string()
                            name="display_name".to_string()
                            r#type="text".to_string()
                            class=FormClass::InputWide.as_str().to_string()
                            data_theme=Theme::Strong
                            value=Signal::derive(move || name.get())
                            on_input=Callback::new(move |ev| set_name.set(event_target_value(&ev)))
                            placeholder="Your name as shown to others".to_string()
                        />
                        <div class=FormClass::ActionsFull.as_str()>
                            <ServerButton
                                class=Signal::derive(|| ButtonClass::Base.as_str().to_string())
                                data_theme=Arc::new(|| Theme::Secondary)
                                r#type="submit".to_string()
                            >
                                {move || if saving.get() { "Saving..." } else { "Save" }}
                            </ServerButton>
                        </div>
                    </div>
                </form>
            </div>
        </section>

        <CenterModal
            show=show_palette
            on_cancel=move || set_show_palette.set(false)
            accent_color=accent_css
        >
            {move || {
                view! {
                    <div class=ClusterClass::Between.as_str()>
                        <h3 class=TextClass::CardTitleTight.as_str()>
                            "Pick an accent"
                        </h3>
                    </div>
                    <h4 class=FormClass::LabelSpaced.as_str()>
                        "Solid"
                    </h4>
                    <div class=FormClass::PaletteGrid.as_str()>
                        {AccentColor::base()
                            .iter()
                            .map(|c| {
                                let color = *c;
                                let name = color.to_string();
                                let var = color.css_var().to_string();
                                view! {
                                    <button
                                        class=ButtonClass::Base.as_str()
                                        data-theme=Theme::Accent.as_str()
                                        style=PaletteStyle::AccentSwatch.with_accent(&var)
                                        title=name.clone()
                                        on:click=move |_| {
                                            set_accent_color.set(color);
                                            set_show_palette.set(false);
                                        }
                                    >
                                        {name.clone()}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>

                    <div class=FormClass::SectionDivider.as_str()></div>

                    <h4 class=FormClass::LabelSpaced.as_str()>
                        "Light"
                    </h4>
                    <div class=FormClass::PaletteGrid.as_str()>
                        {AccentColor::light()
                            .iter()
                            .map(|c| {
                                let color = *c;
                                let name = color.to_string();
                                let var = color.css_var().to_string();
                                view! {
                                    <button
                                        class=ButtonClass::Base.as_str()
                                        data-theme=Theme::Accent.as_str()
                                        style=PaletteStyle::AccentSwatch.with_accent(&var)
                                        title=name.clone()
                                        on:click=move |_| {
                                            set_accent_color.set(color);
                                            set_show_palette.set(false);
                                        }
                                    >
                                        {name.clone()}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                }
            }}
        </CenterModal>
    }
}
