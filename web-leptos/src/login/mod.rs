use leptos::prelude::*;
use leptos::{IntoView, component};

use crate::app_state::{AppStateReader, AppStateWriter};
use peer_practice_shared::user::UserId;

pub mod email;
pub mod pin;

#[component]
pub fn LoginRoute(
    state: AppStateReader,
    write_state: AppStateWriter,
    first_attempt_completed: WriteSignal<bool>,
) -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    navigate("/login", Default::default());
    let (read_step, write_step) = signal(LoginStep::Email);

    let (read_user_id, write_user_id) = signal::<Option<UserId>>(None);

    let on_email_submitted = move |email: String| {
        write_step.set(LoginStep::Pin { email });
    };

    let on_pin_back = move |_| write_step.set(LoginStep::Email);

    let on_pin_success = move |_| {};

    let on_email_submitted = Callback::new(on_email_submitted);
    let on_pin_back = Callback::new(on_pin_back);
    let on_pin_success = Callback::new(on_pin_success);

    view! {
        <div class="w-full" style="margin-top: 1.25rem;">
            <div
                class="w-full"
                style="min-height: 60vh; display: flex; align-items: center; justify-content: center;"
            >
                <div
                    class="w-full max-w-md rounded-xl shadow-lg p-6"
                    style="background: var(--bg-weakest-color); color: var(--bg-base-text);"
                >
                    {move || match read_step.get() {
                        LoginStep::Email => {
                            view! { <email::LoginEmailStep on_email_submitted write_user_id /> }
                                .into_any()
                        }
                        LoginStep::Pin { ref email } => {
                            view! {
                                <pin::LoginPinStep
                                    email=email.clone()
                                    on_pin_back
                                    on_pin_success
                                    read_user_id
                                    state
                                    write_state
                                    first_attempt_completed
                                />
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoginStep {
    Email,
    Pin { email: String },
}
