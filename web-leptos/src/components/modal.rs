use crate::components::buttons::ServerButton;
use leptos::callback::Callback;
use leptos::prelude::*;
use leptos::{IntoView, component};
use std::sync::Arc;

#[component]
pub fn ConfirmDangerousModal(
    #[prop(into)] show: ReadSignal<bool>,
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] message: Option<String>,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
) -> impl IntoView {
    let title = title.unwrap_or_else(|| "Are you sure?".to_string());
    let message = message.unwrap_or_else(|| "This action cannot be undone.".to_string());
    view! {
        <Show when=move || show.get()>
            <div
                role="presentation"
                data-dialog-overlay
                on:click=move |_| {
                    on_cancel.run(());
                }
            >
                <div
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="confirm-title"
                    aria-describedby="confirm-desc"
                    class="card dialog dialog-sheet surface"
                    data-accent="strong"
                    style="--accent: var(--maroon, #be123c); max-height: 90dvh; overflow: auto;"
                    data-scheme="base"
                    on:click=|ev| ev.stop_propagation()
                >
                    <h1 id="confirm-title" class="dialog-title">
                        {title.clone()}
                    </h1>
                    <p id="confirm-desc" class="dialog-desc">
                        {message.clone()}
                    </p>

                    <div class="dialog-actions">
                        <button
                            class="btn"
                            data-theme="secondary"
                            type="button"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                on_cancel.run(())
                            }
                        >
                            "Cancel"
                        </button>
                        <ServerButton
                            class=Signal::derive(|| "btn".to_string())
                            data_theme=Arc::new(|| "danger")
                            r#type="button".to_string()
                            on_click=Callback::new({ move |_| on_confirm.run(()) })
                        >
                            "Delete"
                        </ServerButton>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn CenterModal(
    #[prop(into)] show: ReadSignal<bool>,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(optional, into)] accent_color: Option<ReadSignal<String>>,
    children: ChildrenFn,
) -> impl IntoView {
    let accent_color = accent_color.unwrap_or_else(|| {
        let (default_accent, _set_default_accent) =
            signal(String::from("var(--bg-strongest-color)"));
        default_accent
    });
    view! {
        <Show when=move || show.get()>
            <div role="presentation" data-dialog-overlay on:click=move |_| { on_cancel.run(()) }>
                <div
                    role="dialog"
                    aria-modal="true"
                    class="card dialog dialog-sheet surface"
                    data-accent="strong"
                    style=move || {
                        format!(
                            "--accent: {}; max-height: 90dvh; overflow: auto;",
                            accent_color.get(),
                        )
                    }
                    data-scheme="base"
                >
                    {children()}
                </div>
            </div>
        </Show>
    }
}
