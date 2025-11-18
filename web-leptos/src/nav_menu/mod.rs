use crate::components::modal::CenterModal;
use leptos::prelude::*;
use leptos::{IntoView, component};

#[component]
pub fn NavMenu() -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let (accent_name, _set_accent_name) = signal(String::from("rosewater"));
    let location = || {
        window()
            .location()
            .pathname()
            .unwrap_or_else(|_| "/".to_string())
    };

    fn nav_link_common_box_style() -> &'static str {
        "display:flex; align-items:center; justify-content:center; width:100%; text-align:center;"
    }

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

    fn nav_link_style(active: bool, accent: &str) -> String {
        format!(
            "{} {}",
            nav_link_common_box_style(),
            nav_link_accent_vars(active, accent)
        )
    }

    let current_page_label = move || match location().as_str() {
        "/" => "Home".to_string(),
        "/settings" => "Settings".to_string(),
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
                class="btn"
                data-theme="accent"
                data-accent-strength="strong"
                style="
                --accent: var(--rosewater);
                --accent-light: var(--rosewater-light);
                display: inline-flex;
                align-items: center;
                gap: 0.5rem;
                height: calc(var(--navbar-height, 48px) * 0.8);
                box-sizing: border-box;
                padding: 0.5rem;
                "
            >
                <span
                    aria-hidden="true"
                    style="display:inline-flex; flex-direction:column; gap:3px;"
                >
                    <span style="width:18px; height:2px; background: currentColor; border-radius:2px;"></span>
                    <span style="width:18px; height:2px; background: currentColor; border-radius:2px;"></span>
                    <span style="width:18px; height:2px; background: currentColor; border-radius:2px;"></span>
                </span>
                <span style="font-weight: 600;">{current_page_label}</span>
            </button>

            <Show when=move || menu_open.get()>
                <CenterModal show=menu_open on_cancel=move || set_menu_open.set(false)>
                    {move || {
                        view! {
                            <div style="display:flex; flex-direction:column; align-items:center; padding: 0.5rem 0; width: 100%;">
                                <h1>Navigation</h1>
                                <a
                                    href="/"
                                    class="btn"
                                    data-theme="accent"
                                    data-accent="base"
                                    style=move || {
                                        let active = &location() == "/";
                                        nav_link_style(active, &accent_name.get())
                                    }
                                >
                                    "Home"
                                </a>
                                <a
                                    href="/settings"
                                    class="btn"
                                    data-theme="accent"
                                    data-accent="base"
                                    style=move || {
                                        let active = &location() == "/settings";
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
