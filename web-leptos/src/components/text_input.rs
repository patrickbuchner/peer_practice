use leptos::callback::Callback;
use leptos::prelude::*;
use leptos::{IntoView, component};

use crate::components::theme::{AccentStrength, Theme};

fn input_class(class: Option<String>) -> String {
    let extra = class.unwrap_or_default().trim().to_string();
    if extra.is_empty() {
        return "input".to_string();
    }
    if extra.split_whitespace().any(|name| name == "input") {
        return extra;
    }
    format!("input {extra}")
}

fn input_style(base_style: String, accent_color: Option<ReadSignal<String>>) -> impl Fn() -> String {
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
pub fn TextInput(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] id: Option<String>,
    #[prop(optional)] name: Option<String>,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] r#type: Option<String>,
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional)] inputmode: Option<String>,
    #[prop(optional)] autocomplete: Option<String>,
    #[prop(optional)] pattern: Option<String>,
    #[prop(optional)] maxlength: Option<String>,
    #[prop(optional)] minlength: Option<String>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional)] autofocus: Option<bool>,
    #[prop(optional)] aria_label: Option<String>,
    #[prop(optional, into)] on_input: Option<Callback<leptos::ev::Event>>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
) -> impl IntoView {
    let class = input_class(class);
    let style = input_style(style.unwrap_or_default(), accent_color);
    let data_theme = data_theme.unwrap_or(Theme::Strong);
    let input_type = r#type.unwrap_or_else(|| "text".to_string());
    let pattern = pattern.unwrap_or_else(|| ".*".to_string());

    view! {
        <input
            class=class
            style=style
            id=id.unwrap_or_default()
            name=name.unwrap_or_default()
            placeholder=placeholder.unwrap_or_default()
            r#type=input_type
            inputmode=inputmode.unwrap_or_default()
            autocomplete=autocomplete.unwrap_or_default()
            pattern=pattern
            maxlength=maxlength.unwrap_or_default()
            minlength=minlength.unwrap_or_default()
            required=required.unwrap_or(false)
            autofocus=autofocus.unwrap_or(false)
            aria-label=aria_label.unwrap_or_default()
            prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
            data-theme=data_theme.as_str()
            data-accent-strength=data_accent_strength.map(|accent| accent.as_str()).unwrap_or("")
            on:input=move |ev| {
                if let Some(cb) = &on_input {
                    cb.run(ev);
                }
            }
        />
    }
}

#[component]
pub fn TextAreaInput(
    #[prop(optional)] class: Option<String>,
    #[prop(optional)] style: Option<String>,
    #[prop(optional)] id: Option<String>,
    #[prop(optional)] name: Option<String>,
    #[prop(optional)] placeholder: Option<String>,
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional)] rows: Option<u32>,
    #[prop(optional)] cols: Option<u32>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional)] autofocus: Option<bool>,
    #[prop(optional)] aria_label: Option<String>,
    #[prop(optional, into)] on_input: Option<Callback<leptos::ev::Event>>,
    #[prop(optional)] data_theme: Option<Theme>,
    #[prop(optional)] data_accent_strength: Option<AccentStrength>,
    #[prop(optional)] accent_color: Option<ReadSignal<String>>,
) -> impl IntoView {
    let class = input_class(class);
    let style = input_style(style.unwrap_or_default(), accent_color);
    let data_theme = data_theme.unwrap_or(Theme::Strong);
    let rows = rows.map(|value| value.to_string());
    let cols = cols.map(|value| value.to_string());

    view! {
        <textarea
            class=class
            style=style
            id=id.unwrap_or_default()
            name=name.unwrap_or_default()
            placeholder=placeholder.unwrap_or_default()
            required=required.unwrap_or(false)
            autofocus=autofocus.unwrap_or(false)
            aria-label=aria_label.unwrap_or_default()
            prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
            data-theme=data_theme.as_str()
            data-accent-strength=data_accent_strength.map(|accent| accent.as_str()).unwrap_or("")
            rows=rows.unwrap_or_default()
            cols=cols.unwrap_or_default()
            on:input=move |ev| {
                if let Some(cb) = &on_input {
                    cb.run(ev);
                }
            }
        />
    }
}
