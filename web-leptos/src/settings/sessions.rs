use crate::app_state::AppStateReader;
use crate::components::buttons::{ButtonPreset, ServerButton};
use crate::components::styles::card::CardClass;
use crate::components::styles::form_class::FormClass;
use crate::components::styles::layout::LayoutClass;
use crate::components::styles::text_class::TextClass;
use crate::components::text_input::TextInput;
use crate::components::theme::{CardShadow, SurfaceTheme, Theme};
use leptos::prelude::*;
use peer_practice_shared::colors::accent_colors::AccentColor;
use peer_practice_shared::colors::semantic_colors::BackgroundColor;
use peer_practice_shared::colors::Color;
use peer_practice_shared::messages::client_to_server::SessionAction;
use peer_practice_shared::messages::ClientToServer;
use peer_practice_shared::sessions::{SessionId, SessionInformation};
use std::collections::HashMap;

#[component]
pub fn Settings(state: AppStateReader) -> impl IntoView {
    let sessions = state.sessions.sessions;
    let current_session_id = state.sessions.current;
    let theme = Theme::Surface(SurfaceTheme::Strong);

    let (drafts, set_drafts) = signal::<HashMap<SessionId, String>>(HashMap::new());

    let on_submit = {
        Callback::new(move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();

            let pending = drafts.get_untracked();
            if pending.is_empty() {
                return;
            }

            for (session_id, description) in pending {
                state.send(ClientToServer::Session(SessionAction::UpdateSession(
                    SessionInformation {
                        session_id,
                        description,
                    },
                )));
            }

            set_drafts.set(HashMap::new());
        })
    };

    view! {
        <section class=LayoutClass::ContainerNarrowPadSm.as_str()>
            <div
                class=CardClass::Base.as_str()
                data-theme=theme.as_str()
                data-shadow=CardShadow::None.as_str()
            >
                <h2 class=TextClass::CardTitle.as_str()>"Sessions"</h2>

                <form on:submit=move |ev| on_submit.run(ev)>
                    <For
                        each=move || sessions.get()
                        key=|(id, _)| *id
                        children=move |(id, _info)| {
                            let is_active = move || current_session_id.get() == Some(id);

                            let value = Signal::derive(move || {
                                if let Some(v) = drafts.get().get(&id).cloned() {
                                    v
                                } else {
                                    sessions
                                        .get()
                                        .get(&id)
                                        .map(|s| s.description.clone())
                                        .unwrap_or_default()
                                }
                            });

                            view! {
                                <div
                                    class="session-item"
                                    style="display: flex; gap: 0.75rem; justify-content: space-between; align-items: stretch; padding: 0.5rem"
                                >
                                    <TextInput
                                        r#type="text".to_string()
                                        class=FormClass::InputWide.as_str().to_string()
                                        data_theme=theme
                                        attr:style=move || {
                                            if is_active() {
                                                format!(
                                                    "flex: 1; border: 2px solid {};",
                                                    Color::Accent(AccentColor::Teal).css_var(),
                                                )
                                            } else {
                                                format!(
                                                    "flex: 1; border: 2px solid {};",
                                                    Color::Background(BackgroundColor::Weakest).css_var(),
                                                )
                                            }
                                        }
                                        value=value
                                        on_input=Callback::new(move |ev| {
                                            let new_value = event_target_value(&ev);
                                            set_drafts.update(|m| {
                                                m.insert(id, new_value);
                                            });
                                        })
                                        placeholder="Your device (eg. Chrome Browser Laptop)".to_string()
                                    />

                                    <ServerButton
                                        preset=ButtonPreset::DangerSessionLogout
                                        r#type="button".to_string()
                                        on_click=Callback::new(move |_| {
                                            state.send(ClientToServer::Session(SessionAction::LogOutSession(id)));
                                        })
                                    >
                                        "Log out"
                                    </ServerButton>
                                </div>
                            }
                        }
                    />

                    <div style="display: flex; justify-content: center; gap: 0.75rem; padding: 0.5rem;">
                        <ServerButton
                            preset=ButtonPreset::Secondary
                            r#type="submit".to_string()
                        >
                            "Save session names"
                        </ServerButton>

                        <ServerButton
                            preset=ButtonPreset::Danger
                            r#type="button".to_string()
                            on_click=Callback::new(move |_| {
                                state.send(ClientToServer::Session(SessionAction::LogOutAllSessions));
                            })
                        >
                            "Log out all"
                        </ServerButton>
                    </div>
                </form>
            </div>
        </section>
    }
}
