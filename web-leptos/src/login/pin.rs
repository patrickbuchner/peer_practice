use leptos::logging::log;
use leptos::prelude::*;

use crate::app_state::{AppStateReader, AppStateWriter};
use crate::host;
use crate::websocket::attempt_connect;
use peer_practice_shared::authentication::login_data::PinLogin;
use peer_practice_shared::user::UserId;

#[component]
pub fn LoginPinStep(
    email: String,
    #[prop(into)] on_pin_back: Callback<()>,
    #[prop(into)] on_pin_success: Callback<String>,
    #[prop(into)] read_user_id: ReadSignal<Option<UserId>>,
    #[prop(into)] state: AppStateReader,
    #[prop(into)] write_state: AppStateWriter,
    #[prop(into)] first_attempt_completed: WriteSignal<bool>,
) -> impl IntoView {
    let (read_pin, write_pin) = signal(String::new());
    let (show_toast_read, show_toast_write) = signal(false);

    let pin_complete = Signal::derive({
        move || {
            let v = read_pin.get();
            v.len() == 6 && v.chars().all(|c| c.is_ascii_digit())
        }
    });

    let submit_or_toast = {
        move || {
            if pin_complete.get() {
                leptos::task::spawn_local({
                    let pin = read_pin.get().clone();
                    let id = read_user_id.read().unwrap();
                    async move {
                        let client = reqwest::Client::new();
                        let payload = PinLogin { pin, id };

                        match client
                            .post(format!("https://{}/v1/pin", host()))
                            .json(&payload)
                            .send()
                            .await
                        {
                            Ok(resp) => {
                                if let Err(e) = resp.error_for_status_ref() {
                                    log!("Login initiation failed (non-2xx): {}", e);
                                } else {
                                    attempt_connect(write_state, state, first_attempt_completed);
                                }
                            }
                            Err(e) => {
                                log!("Network error while initiating login: {}", e);
                            }
                        }
                    }
                });
                on_pin_success.run(read_pin.read().clone());
            } else {
                show_toast_write.set(true);
            }
        }
    };

    view! {
        <div class="space-y-4 relative">
            <h2 class="text-xl font-semibold">"Enter the 6‑digit code"</h2>
            <p class="text-sm opacity-90">"We sent a code to:"</p>
            <p class="text-sm font-mono opacity-90">{email.clone()}</p>

            <form
                class="mt-3 space-y-4"
                on:submit=move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default();
                    submit_or_toast();
                }
            >
                <input
                    type="text"
                    inputmode="numeric"
                    pattern="[0-9]*"
                    maxlength="6"
                    class="w-full px-3 py-2 rounded-md outline-none text-center \
                    bg-[var(--bg-weak-color)] text-[var(--bg-base-text)]"
                    placeholder="6-digit code"
                    prop:value=Signal::derive({ move || read_pin.get() })
                    autofocus=true
                    on:input=move |ev| {
                        let cleaned: String = event_target_value(&ev)
                            .chars()
                            .filter(|ch| ch.is_ascii_digit())
                            .take(6)
                            .collect();
                        write_pin.set(cleaned);
                        if show_toast_read.get() {
                            show_toast_write.set(false);
                        }
                    }
                />

                <div class="mt-4 flex items-center justify-between gap-2">
                    <button
                        type="button"
                        class="px-4 py-2 rounded-md font-medium transition-colors \
                        bg-[var(--secondary-weak-color)] text-[var(--secondary-weak-text)] hover:opacity-90"
                        on:click=move |_| {
                            write_pin.set(String::new());
                            show_toast_write.set(false);
                            on_pin_back.run(());
                        }
                    >
                        "Back"
                    </button>
                    <button
                        type="submit"
                        class=Signal::derive({
                            move || {
                                let base = "px-4 py-2 rounded-md font-medium transition-colors ";
                                if pin_complete.get() {
                                    format!(
                                        "{base}bg-[var(--primary-base-color)] text-[var(--primary-base-text)] hover:opacity-90",
                                    )
                                } else {
                                    format!(
                                        "{base}bg-[var(--bg-strong-color)] text-[var(--bg-strong-text)] opacity-50",
                                    )
                                }
                            }
                        })
                        aria-disabled=Signal::derive(move || {
                            if pin_complete.get() { "false" } else { "true" }
                        })
                    >
                        "Next"
                    </button>
                </div>
            </form>

            {move || {
                if show_toast_read.get() {
                    view! {
                        <div class="toast" role="alert">
                            "Please enter the full 6-digit code"
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </div>
    }
}
