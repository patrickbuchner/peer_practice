use crate::components::modal::CenterModal;
use crate::components::styles::button_class::ButtonClass;
use crate::components::styles::nav_menu::NavMenuClass;
use crate::components::styles::navbar::nav_link_style;
use crate::components::theme::{AccentStrength, Theme};
use leptos::prelude::*;
use leptos::{IntoView, component};
use leptos_router::hooks::use_location;
use peer_practice_shared::colors::accent_colors::AccentColor;

#[component]
pub fn NavMenu() -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let (accent_name, _set_accent_name) = signal(AccentColor::Rosewater);
    let location = use_location();
    let current_path = move || location.pathname.get();

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
                class=ButtonClass::NavMenu.as_str()
                data-theme=Theme::Accent.as_str()
                data-accent-strength=AccentStrength::Strong.as_str()
            >
                <span aria-hidden="true" class=NavMenuClass::Icon.as_str()>
                    <span class=NavMenuClass::IconBar.as_str()></span>
                    <span class=NavMenuClass::IconBar.as_str()></span>
                    <span class=NavMenuClass::IconBar.as_str()></span>
                </span>
                <span class=NavMenuClass::Label.as_str()>{current_page_label}</span>
            </button>

            <Show when=move || menu_open.get()>
                <CenterModal show=menu_open on_cancel=move || set_menu_open.set(false)>
                    {move || {
                        view! {
                            <div class=NavMenuClass::Panel.as_str()>
                                <h1>Navigation</h1>
                                <a
                                    href="/"
                                    class=ButtonClass::NavMenuLink.as_str()
                                    data-theme=Theme::Accent.as_str()
                                    data-accent=AccentStrength::Base.as_str()
                                    style=move || {
                                        let active = current_path() == "/";
                                        nav_link_style(active, &accent_name.get())
                                    }
                                >
                                    "Home"
                                </a>
                                <a
                                    href="/settings"
                                    class=ButtonClass::NavMenuLink.as_str()
                                    data-theme=Theme::Accent.as_str()
                                    data-accent=AccentStrength::Base.as_str()
                                    style=move || {
                                        let active = current_path() == "/settings";
                                        nav_link_style(active, &accent_name.get())
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
