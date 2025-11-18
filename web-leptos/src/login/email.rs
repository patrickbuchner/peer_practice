use leptos::logging::log;
use leptos::prelude::*;

use crate::host;
use peer_practice_shared::authentication::login_data::LoginData;
use peer_practice_shared::authentication::method::AuthenticationMethod;
use peer_practice_shared::email::Email;
use peer_practice_shared::user::UserId;

#[component]
pub fn LoginEmailStep(
    #[prop(into)] on_email_submitted: Callback<String>,
    #[prop(into)] write_user_id: WriteSignal<Option<UserId>>,
) -> impl IntoView {
    let (email_read, email_write) = signal(String::new());

    let on_submit = {
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            leptos::task::spawn_local({
                let email_clone = email_read.get().clone();
                log!("Email clone: {}", email_clone);
                async move {
                    let client = reqwest::Client::new();
                    let payload = LoginData {
                        // Assuming Email implements Into from String in the shared crate
                        email: Email::new(&email_clone).unwrap(),
                        auth: AuthenticationMethod::EmailOTP,
                    };

                    log!("Initiating login with email: {}", email_clone);

                    match client
                        .post(format!("https://{}/v1/login", host()))
                        .json(&payload)
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            if let Err(e) = resp.error_for_status_ref() {
                                log!("Login initiation failed (non-2xx): {}", e);
                            } else {
                                let val = resp.json::<Option<UserId>>().await;
                                log!("Login initiation succeeded: {:?}", val);
                                if let Ok(id) = val {
                                    write_user_id.set(id);
                                }
                            }
                        }
                        Err(e) => {
                            log!("Network error while initiating login: {}", e);
                        }
                    }
                    on_email_submitted.run(email_read.get_untracked());
                }
            });
        }
    };

    view! {
        <form class="space-y-4" on:submit=on_submit>
            <h2 class="text-xl font-semibold">"Log in"</h2>
            <p class="text-sm opacity-90">"What's your email?"</p>
            <div class="mt-2">
                <input
                    type="email"
                    required
                    class="w-full px-3 py-2 rounded-md outline-none text-center"
                    style="background: var(--bg-weak-color); color: var(--bg-base-text);"
                    placeholder="you@example.com"
                    prop:value=Signal::derive(move || email_read.get())
                    autofocus=true
                    on:input=move |ev| email_write.set(event_target_value(&ev).trim().to_string())
                />
            </div>
            <div class="mt-4 flex gap-2 justify-end">
                <button
                    type="submit"
                    class="px-4 py-2 rounded-md font-medium transition-colors \
                    bg-[var(--primary-base-color)] text-[var(--primary-base-text)] hover:opacity-90"
                >
                    "Next"
                </button>
            </div>
        </form>
    }
}
