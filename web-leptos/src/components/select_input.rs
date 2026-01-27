use leptos::callback::Callback;
use leptos::prelude::*;
use leptos::{IntoView, component};

use crate::components::theme::{AccentStrength, Theme};

fn select_class(class: Option<String>) -> String {
    let extra = class.unwrap_or_default().trim().to_string();
    if extra.is_empty() {
        return "combo".to_string();
    }
    if extra.split_whitespace().any(|name| name == "combo") {
        return extra;
    }
    format!("combo {extra}")
}

fn select_style(base_style: String, accent_color: Option<ReadSignal<String>>) -> impl Fn() -> String {
    move || {
        let mut style_value = base_style.clone();
        if let Some(accent) = accent_color.as_ref() {
            if style_value.trim().is_empty() {
                style_value = format!("--accent: {};", accent.get());
            } else {
                let trimmed = style_value.trim_end_matches(';');
                style_value = format!("{trimmed}; --accent: {};", accent.get());
            }
        }
        style_value
    }
}

#[component]
pub fn SelectInput(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] name: Option<String>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional)] multiple: Option<bool>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional, into)] on_change: Option<Callback<leptos::ev::Event>>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
    children: Children,
) -> impl IntoView {
    let class = select_class(class);
    let style = select_style(style.unwrap_or_default(), accent_color);
    let data_theme = data_theme.unwrap_or(Theme::Strong);

    view! {
        <select
            class=class
            style=style
            name=name.unwrap_or_default()
            required=required.unwrap_or(false)
            multiple=multiple.unwrap_or(false)
            disabled=disabled.unwrap_or(false)
            prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
            data-theme=data_theme.as_str()
            data-accent-strength=data_accent_strength.map(|accent| accent.as_str()).unwrap_or("")
            on:change=move |ev| {
                if let Some(cb) = &on_change {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </select>
    }
}
