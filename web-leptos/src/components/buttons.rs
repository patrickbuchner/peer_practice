use crate::app_state::AppStateReader;
use crate::components::modal::ConfirmDangerousModal;
use leptos::callback::Callback;
use leptos::prelude::*;
use leptos::{IntoView, component};
use std::sync::Arc;

#[component]
pub fn ServerButton(
    #[prop(optional)] class: Option<Signal<String>>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] title: Option<String>,
    #[prop(optional)] aria_label: Option<String>,
    #[prop(optional)] r#type: Option<String>,
    #[prop(optional, into)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] data_theme: Option<Arc<dyn Fn() -> &'static str + Send + Sync>>,
    children: Children,
) -> impl IntoView {
    let state = expect_context::<AppStateReader>();
    let connected = Signal::derive(move || state.connected_to_server());
    let externally_disabled = disabled.unwrap_or_else(|| Signal::derive(|| false));
    let is_disabled = Signal::derive(move || !connected.get() || externally_disabled.get());

    let btn_type = r#type.unwrap_or_else(|| "button".to_string());
    let aria = aria_label.unwrap_or_default();
    let style = style.unwrap_or_default();
    let data_theme = data_theme.unwrap_or_else(|| Arc::new(|| "primary"));

    let guarded_on_click = on_click.map(|cb| {
        Callback::new(move |ev: leptos::ev::MouseEvent| {
            if !is_disabled.get_untracked() {
                cb.run(ev);
            } else {
                ev.prevent_default();
                ev.stop_propagation();
            }
        })
    });

    view! {
        <button
            class=class
            style=style
            title=title.unwrap_or_default()
            aria-label=aria
            r#type=btn_type
            disabled=move || is_disabled.get()
            aria-disabled=move || if is_disabled.get() { "true" } else { "false" }
            data-theme=move || data_theme()
            on:click=move |ev| {
                if let Some(cb) = &guarded_on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn ConfirmDeleteButton(
    #[prop(optional)] button_label: Option<String>,
    #[prop(optional)] button_title: Option<String>,
    #[prop(optional)] aria_label: Option<String>,
    #[prop(optional)] confirm_title: Option<String>,
    #[prop(optional)] confirm_message: Option<String>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    let (show_confirm, set_show_confirm) = signal(false);

    let btn_label = button_label.unwrap_or_else(|| "🗑️".to_string());
    let btn_title = button_title.unwrap_or_else(|| "Delete".to_string());
    let aria = aria_label.unwrap_or_else(|| "Delete".to_string());
    let dialog_title = confirm_title.unwrap_or_else(|| "Are you sure?".to_string());
    let dialog_message =
        confirm_message.unwrap_or_else(|| "This action cannot be undone.".to_string());

    view! {
        <>
            <button
                class="btn"
                type="button"
                aria-label=aria
                title=btn_title
                style="font-size: .9rem; padding: .35rem .6rem;"
                data-theme="danger"
                on:click=move |_| {
                    set_show_confirm.set(true);
                }
            >
                {btn_label}
            </button>

            <ConfirmDangerousModal
                show=show_confirm
                title=dialog_title.clone()
                message=dialog_message.clone()
                on_confirm=Callback::new({
                    move |_| {
                        on_confirm.run(());
                        set_show_confirm.set(false);
                    }
                })
                on_cancel=Callback::new({ move |_| set_show_confirm.set(false) })
            />
        </>
    }
}
