use leptos::prelude::*;
use leptos::{IntoView, component};

use crate::components::theme::{AccentStrength, Theme};

fn text_box_class(class: Option<String>) -> String {
    let extra = class.unwrap_or_default().trim().to_string();
    let mut classes = vec!["markdown-body".to_string(), "surface".to_string()];
    if !extra.is_empty() {
        for name in extra.split_whitespace() {
            if classes.iter().any(|existing| existing == name) {
                continue;
            }
            classes.push(name.to_string());
        }
    }
    classes.join(" ")
}

fn text_box_style(base_style: String, accent_color: Option<ReadSignal<String>>) -> impl Fn() -> String {
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

fn surface_class(class: Option<String>) -> String {
    let extra = class.unwrap_or_default().trim().to_string();
    if extra.is_empty() {
        return "surface".to_string();
    }
    if extra.split_whitespace().any(|name| name == "surface") {
        return extra;
    }
    format!("surface {extra}")
}

#[component]
pub fn TextBox(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] role: Option<String>,
    #[prop(optional)] aria_label: Option<String>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
    #[prop(into)] html: Signal<String>,
) -> impl IntoView {
    let class = text_box_class(class);
    let style = text_box_style(style.unwrap_or_default(), accent_color);

    view! {
        <div
            class=class
            style=style
            role=role.unwrap_or_default()
            aria-label=aria_label.unwrap_or_default()
            data-theme=data_theme.map(|theme| theme.as_str()).unwrap_or("")
            data-accent-strength=data_accent_strength.map(|accent| accent.as_str()).unwrap_or("")
            inner_html=html
        />
    }
}

#[component]
pub fn SurfaceBox(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
    children: Children,
) -> impl IntoView {
    let class = surface_class(class);
    let style = text_box_style(style.unwrap_or_default(), accent_color);

    view! {
        <div
            class=class
            style=style
            data-theme=data_theme.map(|theme| theme.as_str()).unwrap_or("")
            data-accent-strength=data_accent_strength.map(|accent| accent.as_str()).unwrap_or("")
        >
            {children()}
        </div>
    }
}
