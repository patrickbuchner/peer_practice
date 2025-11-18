use leptos::prelude::*;
use std::sync::Arc;

use crate::app_state::AppStateReader;
use crate::components::buttons::ServerButton;
use crate::components::modal::CenterModal;
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
        state.send(ClientToServer::UpdateUser(UserDisplay {
            id,
            display_name: Some(new_name.clone()),
        }));

        set_saving.set(false);
    };

    view! {
        <section class="container container-narrow pad-sm">
            <div class="card">
                <h2 class="card-title">"Settings"</h2>
                <form class="form" style="margin-top: 1rem;" on:submit=on_submit>
                    <div
                        class="grid"
                        style="display: grid; grid-template-columns: max-content 1fr; column-gap: .75rem; row-gap: .5rem; align-items: center;"
                    >
                        <label for="display_name" class="label" style="justify-self: end;">
                            "Display name"
                        </label>
                        <input
                            id="display_name"
                            name="display_name"
                            type="text"
                            class="w-full"
                            data-theme="base"
                            data-accent=""
                            data-accent-strength="base"
                            style="--accent: var(--bg-strongest-color); padding: .6rem .75rem; border-radius: .6rem; border: 1px solid currentColor; min-width: 20rem;"
                            value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                            placeholder="Your name as shown to others"
                        />
                        //
                        // <label class="label" style="justify-self: end;">
                        //     "Accent color"
                        // </label>
                        // <div class="cluster" style="--cluster-justify: flex-start;">
                        //     <button
                        //         class="btn"
                        //         data-theme="accent"
                        //         style=move || {
                        //             format!("--accent: {}; min-width: 9rem;", accent_css.get())
                        //         }
                        //         on:click=move |ev| {
                        //             ev.prevent_default();
                        //             set_show_palette.set(true);
                        //         }
                        //         title=Signal::derive(move || format!("{}", accent_color.get()))
                        //     >
                        //         {move || format!("Choose color ({})", accent_color.get())}
                        //     </button>
                        // </div>

                        <div
                            class="actions actions-inline gap-sm align-center"
                            style="grid-column: 1 / -1; margin-top: .25rem;"
                        >
                            <ServerButton
                                class=Signal::derive(|| "btn".to_string())
                                data_theme=Arc::new(|| "secondary")
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
                    <div
                        class="cluster"
                        style="--cluster-justify: space-between; margin-bottom: .5rem;"
                    >
                        <h3 class="card-title" style="margin: 0;">
                            "Pick an accent"
                        </h3>
                    </div>
                    <h4 class="label" style="margin: .25rem 0;">
                        "Solid"
                    </h4>
                    <div style="
                    display: grid;
                    grid-template-columns: repeat(4, minmax(0, 1fr));
                    gap: .5rem;
                    ">
                        {AccentColor::base()
                            .iter()
                            .map(|c| {
                                let color = *c;
                                let name = color.to_string();
                                let var = color.css_var().to_string();
                                view! {
                                    <button
                                        class="btn"
                                        data-theme="accent"
                                        style=format!("--accent: {}; width: 100%;", var)
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

                    <div style="border-top: 1px solid var(--border-color, currentColor); opacity: .25; margin: .75rem 0;"></div>

                    <h4 class="label" style="margin: .25rem 0;">
                        "Light"
                    </h4>
                    <div style="
                    display: grid;
                    grid-template-columns: repeat(4, minmax(0, 1fr));
                    gap: .5rem;
                    ">
                        {AccentColor::light()
                            .iter()
                            .map(|c| {
                                let color = *c;
                                let name = color.to_string();
                                let var = color.css_var().to_string();
                                view! {
                                    <button
                                        class="btn"
                                        data-theme="accent"
                                        style=format!("--accent: {}; width: 100%;", var)
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
