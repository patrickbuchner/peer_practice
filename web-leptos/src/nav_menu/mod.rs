use crate::components::modal::CenterModal;
use leptos::prelude::*;
use crate::components::theme::{AccentStrength, Theme};
use leptos::{IntoView, component};
use leptos_router::hooks::use_location;

#[component]
pub fn NavMenu() -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let (accent_name, _set_accent_name) = signal(String::from("rosewater"));
    let location = use_location();
    let current_path = move || location.pathname.get();

    fn nav_link_accent_vars(active: bool, accent: &str) -> String {
        if active {
            format!("--accent: var(--{0}); --accent-light: var(--{0});", accent)
        } else {
            format!(
                "--accent: var(--{0}-light); --accent-light: var(--{0}-light);",
                accent
            )
        }
    }

    let current_page_label = move || match current_path().as_str() {
        "/" => "Home".to_string(),
        "/settings" => "Settings".to_string(),
        path if path.starts_with("/chat/") => "Chat".to_string(),
        other => {
            let seg = other.trim_end_matches('/').rsplit('/').next().unwrap_or("");
            if seg.is_empty() {
                "Home".to_string()
            } else {
                let mut ch = seg.chars();
                match ch.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + ch.as_str(),
                    None => seg.to_string(),
                }
            }
        }
    };

    view! {
        <>
            <button
                aria-label="Open navigation menu"
                title="Menu"
                on:click=move |_| set_menu_open.set(true)
                class="btn nav-menu-button"
                data-theme=Theme::Accent.as_str()
                data-accent-strength=AccentStrength::Strong.as_str()
            >
                <span aria-hidden="true" class="nav-menu-icon">
                    <span class="nav-menu-icon-bar"></span>
                    <span class="nav-menu-icon-bar"></span>
                    <span class="nav-menu-icon-bar"></span>
                </span>
                <span class="nav-menu-label">{current_page_label}</span>
            </button>

            <Show when=move || menu_open.get()>
                <CenterModal show=menu_open on_cancel=move || set_menu_open.set(false)>
                    {move || {
                        view! {
                            <div class="nav-menu-panel">
                                <h1>Navigation</h1>
                                <a
                                    href="/"
                                    class="btn nav-menu-link"
                                    data-theme=Theme::Accent.as_str()
                                    data-accent=AccentStrength::Base.as_str()
                                    style=move || {
                                        let active = current_path() == "/";
                                        nav_link_accent_vars(active, &accent_name.get())
                                    }
                                >
                                    "Home"
                                </a>
                                <a
                                    href="/settings"
                                    class="btn nav-menu-link"
                                    data-theme=Theme::Accent.as_str()
                                    data-accent=AccentStrength::Base.as_str()
                                    style=move || {
                                        let active = current_path() == "/settings";
                                        nav_link_accent_vars(active, &accent_name.get())
                                    }
                                >
                                    "Settings"
                                </a>
                            </div>
                        }
                    }}
                </CenterModal>
            </Show>
        </>
    }
}
