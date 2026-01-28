use leptos::logging::log;
use leptos::prelude::*;
use crate::components::styles::{ButtonClass, FormClass, LayoutClass, TextClass};
use crate::components::text_input::TextInput;
use crate::components::theme::Theme;

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
        <form class=LayoutClass::StackSm.as_str() on:submit=on_submit>
            <h2 class=TextClass::Lg.as_str()>"Log in"</h2>
            <p class=TextClass::SmMuted.as_str()>"What's your email?"</p>
            <div>
                <TextInput
                    r#type="email".to_string()
                    required=true
                    class=FormClass::InputCenter.as_str().to_string()
                    placeholder="you@example.com".to_string()
                    value=Signal::derive(move || email_read.get())
                    autofocus=true
                    on_input=Callback::new(move |ev| {
                        email_write.set(event_target_value(&ev).trim().to_string());
                    })
                />
            </div>
            <div class=LayoutClass::RowEnd.as_str()>
                <button
                    type="submit"
                    class=ButtonClass::Base.as_str()
                    data-theme=Theme::Primary.as_str()
                >
                    "Next"
                </button>
            </div>
        </form>
    }
}
