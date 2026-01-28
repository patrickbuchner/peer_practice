use leptos::callback::Callback;
use leptos::prelude::*;
use leptos::{IntoView, component};

use crate::components::styles::{CardShellClass, ShadowColor};
use crate::components::theme::{AccentStrength, CardShadow, Theme};

fn card_class(class: Option<String>) -> String {
    let extra = class.unwrap_or_default().trim().to_string();
    if extra.is_empty() {
        return "card".to_string();
    }
    if extra.split_whitespace().any(|name| name == "card") {
        return extra;
    }
    format!("card {extra}")
}

fn card_style(base_style: String, accent_color: Option<ReadSignal<String>>) -> impl Fn() -> String {
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
pub fn Card(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_shadow: Option<CardShadow>,
    #[prop(optional)] shadow_color: Option<ReadSignal<ShadowColor>>,
    #[prop(optional)] data_accent: Option<AccentStrength>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
    children: Children,
) -> impl IntoView {
    let class = card_class(class);
    let style = card_style(style.unwrap_or_default(), accent_color);
    let data_theme = data_theme.unwrap_or(Theme::Strong);
    let data_shadow = data_shadow.unwrap_or(CardShadow::Weakest);
    let shadow_color = shadow_color.unwrap_or_else(|| {
        let (read, _set) = signal(ShadowColor::Base);
        read
    });

    view! {
        <div class=CardShellClass::Base.as_str()>
            <div
                class=class
                style=style
                data-theme=data_theme.as_str()
                data-shadow=data_shadow.as_str()
                data-shadow-color=move || shadow_color.get().as_str()
                attr:data-accent=move || data_accent.map(|accent| accent.as_str())
                attr:data-accent-strength=move || data_accent_strength.map(|accent| accent.as_str())
            >
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn CardForm(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_shadow: Option<CardShadow>,
    #[prop(optional)] shadow_color: Option<ReadSignal<ShadowColor>>,
    #[prop(optional)] data_accent: Option<AccentStrength>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
    #[prop(optional, into)] on_submit: Option<Callback<leptos::ev::SubmitEvent>>,
    children: Children,
) -> impl IntoView {
    let class = card_class(class);
    let style = card_style(style.unwrap_or_default(), accent_color);
    let data_theme = data_theme.unwrap_or(Theme::Strong);
    let data_shadow = data_shadow.unwrap_or(CardShadow::Weakest);
    let shadow_color = shadow_color.unwrap_or_else(|| {
        let (read, _set) = signal(ShadowColor::Base);
        read
    });

    view! {
        <div class=CardShellClass::Base.as_str()>
            <form
                class=class
                style=style
                data-theme=data_theme.as_str()
                data-shadow=data_shadow.as_str()
                data-shadow-color=move || shadow_color.get().as_str()
                attr:data-accent=move || data_accent.map(|accent| accent.as_str())
                attr:data-accent-strength=move || data_accent_strength.map(|accent| accent.as_str())
                on:submit=move |ev| {
                    if let Some(cb) = &on_submit {
                        cb.run(ev);
                    }
                }
            >
                {children()}
            </form>
        </div>
    }
}
